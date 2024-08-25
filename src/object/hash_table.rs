use std::mem::size_of;

use rand::Rng;

use crate::{memory::Address, object::prelude::Ptr, thread::ThreadPtr};

use super::{prelude::JInt, VMObject};

pub type HashTablePtr = Ptr<HashTable>;

pub trait GetEntryWithKey<K> {
    fn hash_key(key: K) -> JInt;

    fn entry_equals_key(value: Address, key: K) -> bool;
}

pub trait InsertNewWithKey<K, R> {
    fn new_entry_with_key(key: K, key_hash: JInt, thread: ThreadPtr) -> Ptr<R>;
}

pub struct HashTable {
    capacity: i32,
    pub size: i32,
    hasher: TableHasher,
}

impl HashTable {
    const DEFAULT_SIZE: usize = 8;
    const ENTRIES_OFFSET: usize = size_of::<HashTable>();

    pub fn new(thread: ThreadPtr) -> HashTablePtr {
        return Self::new_with_init_size(Self::DEFAULT_SIZE as i32, thread);
    }

    pub fn new_with_init_size(init_size: i32, thread: ThreadPtr) -> HashTablePtr {
        let capacity = init_size / 3 * 4;
        let capacity = next_prime(capacity as u64) as i32;
        let mut table = HashTablePtr::from_addr(
            thread
                .heap()
                .alloc_obj_permanent(Self::object_size(capacity)),
        );
        table.capacity = capacity;
        table.size = 0;
        table.hasher = Self::get_hasher(capacity);
        return table;
    }

    fn object_size(capacity: i32) -> usize {
        return Self::ENTRIES_OFFSET + size_of::<Address>() * capacity as usize;
    }

    #[must_use]
    pub fn insert<V>(&mut self, val: Ptr<V>, thread: ThreadPtr) -> HashTablePtr
    where
        V: VMObject,
    {
        let entry = self.probe(V::hash(val.cast()), |entry: Ptr<V>| {
            V::equals(entry.cast(), val.cast())
        });
        return self.insert_entry(entry, val, thread);
    }

    pub fn get_value_by_str<K, V>(&self, key: K) -> Option<Ptr<V>>
    where
        K: Copy,
        V: VMObject + GetEntryWithKey<K>,
    {
        let entry = self.probe(V::hash_key(key), |entry: Ptr<V>| {
            V::entry_equals_key(entry.as_address(), key)
        });
        return if (*entry).is_not_null() {
            Some(*entry)
        } else {
            None
        };
    }

    pub fn get_value_by_str_unchecked<K, V>(&self, key: K) -> Ptr<V>
    where
        K: Copy,
        V: VMObject + GetEntryWithKey<K>,
    {
        let entry = self.probe(V::hash_key(key), |entry: Ptr<V>| {
            V::entry_equals_key(entry.as_address(), key)
        });
        return *entry;
    }

    #[must_use]
    pub fn get_or_insert_str<K, V>(&mut self, key: K, thread: ThreadPtr) -> (HashTablePtr, Ptr<V>)
    where
        K: Copy,
        V: VMObject + GetEntryWithKey<K> + InsertNewWithKey<K, V>,
    {
        let key_hash = V::hash_key(key);
        let entry = self.probe(key_hash, |entry: Ptr<V>| {
            V::entry_equals_key(entry.as_address(), key)
        });
        let mut table: Ptr<HashTable> = Ptr::from_ref(self);
        let mut value = *entry;
        if value.is_null() {
            value = V::new_entry_with_key(key, key_hash, thread);
            table = self.insert_entry(entry, value, thread);
        }
        return (table, value);
    }

    pub fn foreach_entries<V, F: Fn(Ptr<V>)>(&self, it: F) {
        if self.size == 0 {
            return;
        }
        let prev_entries: Ptr<Ptr<V>> = self.entries();
        let mut num_iter = 0;
        for index in 0..self.capacity {
            let entry = prev_entries.offset(index as isize);
            if (*entry).is_not_null() {
                it(*entry);
                num_iter += 1;

                if num_iter == self.size {
                    break;
                }
            }
        }
    }

    pub fn mut_foreach_entries<V, F: FnMut(Ptr<V>)>(&self, mut it: F) {
        if self.size == 0 {
            return;
        }
        let prev_entries: Ptr<Ptr<V>> = self.entries();
        let mut num_iter = 0;
        for index in 0..self.capacity {
            let entry = prev_entries.offset(index as isize);
            if (*entry).is_not_null() {
                it(*entry);
                num_iter += 1;

                if num_iter == self.size {
                    break;
                }
            }
        }
    }

    #[must_use]
    fn insert_entry<V>(
        &mut self,
        mut entry: Ptr<Ptr<V>>,
        val: Ptr<V>,
        thread: ThreadPtr,
    ) -> HashTablePtr
    where
        V: VMObject,
    {
        // log::trace!("insert_entry self: {:x} entry: {:x}, *entry: {:x}", HashTablePtr::from_ref(self).as_usize(), entry.as_usize(), (*entry).as_usize());
        if (*entry).is_null() {
            let table = Ptr::from_ref(self);
            if (self.size + 1) as f32 / self.capacity as f32 >= 0.75 {
                let mut new_table = HashTable::new_with_init_size(self.size << 2, thread);

                let prev_entries: Ptr<Ptr<V>> = self.entries();
                let mut prev_num_iter = 0;
                for index in 0..self.capacity {
                    let prev_entry = prev_entries.offset(index as isize);
                    if (*prev_entry).is_not_null() {
                        new_table = new_table.insert(*prev_entry, thread);
                        prev_num_iter += 1;

                        if prev_num_iter == self.size {
                            break;
                        }
                    }
                }
                debug_assert_eq!(prev_num_iter, self.size);
                return new_table.insert(val, thread);
            }
            *entry = val;
            self.size += 1;

            let mut num_it = 0;
            self.mut_foreach_entries(|_: Ptr<V>| {
                num_it += 1;
            });
            debug_assert_eq!(num_it, self.size);
            return table;
        } else {
            *entry = val;
            return Ptr::from_ref(self);
        }
    }

    fn entries<V>(&self) -> Ptr<Ptr<V>> {
        Ptr::from_ref_offset_bytes(self, Self::ENTRIES_OFFSET as isize)
    }

    fn probe<V, EqFn: Fn(Ptr<V>) -> bool>(&self, val_hash: i32, equals_fn: EqFn) -> Ptr<Ptr<V>> {
        let origin_offset = self.hasher.hash(val_hash, self.capacity);
        let mut offset = origin_offset;
        let mut probe_count = 0;
        loop {
            let entry = self.entries::<V>().offset(offset as isize);
            let entry_val = *entry;
            if entry_val.is_null() || equals_fn(entry_val) {
                return entry;
            }
            probe_count += 1;
            if probe_count % 2 != 0 {
                offset = origin_offset + (probe_count + 1) / 2 * (probe_count + 1) / 2;
                while offset >= self.capacity {
                    offset -= self.capacity;
                }
            } else {
                offset = origin_offset - probe_count / 2 * probe_count / 2;
                while offset < 0 {
                    offset += self.capacity;
                }
            }
        }
    }

    fn get_hasher(capacity: i32) -> TableHasher {
        let p = next_prime(capacity as u64);
        let mut rng = rand::thread_rng();
        let a: u64 = rng.gen_range(1..p);
        let b: u64 = rng.gen_range(0..p);
        return TableHasher { a, b, p };
    }
}

fn next_prime(mut n: u64) -> u64 {
    if n <= 2 {
        return 2;
    }
    if n % 2 == 0 {
        n += 1;
    }
    while !is_prime(n) {
        n += 2;
    }
    n
}

fn is_prime(n: u64) -> bool {
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 || n <= 1 {
        return false;
    }
    let lower = 3;
    let upper = sqrt(n);
    (lower..(upper + 1))
        .step_by(2)
        .all(|maybe_divisor| n % maybe_divisor != 0)
}

fn sqrt(n: u64) -> u64 {
    let (mut low, mut high) = (1, n);
    let mut mid = (low + high) / 2;
    while low < high {
        mid = (low + high) / 2;
        let square = mid * mid;
        if square == n {
            return mid;
        } else if square > n {
            high = mid - 1
        } else {
            low = mid + 1
        }
    }
    if mid * mid == n {
        mid
    } else {
        high
    }
}

struct TableHasher {
    a: u64,
    b: u64,
    p: u64,
}

impl TableHasher {
    fn hash(&self, val: i32, capacity: i32) -> i32 {
        return (((self.a * val as u64 + self.b) % self.p) % (capacity as u64)) as i32;
    }
}
