package rsvm;

public class MethodCall {
    public static int fibonacci(int num) {
        if (num == 1 || num == 2) {
            return 1;
        }
        return fibonacci(num - 1) + fibonacci(num - 2);
    }

    public static String invokeVirtual() {
        Base b = new Sub();
        String name = b.getName();
        return name;
    }

    static class Base {
        public String getName() {
            return "Base";
        }
    }

    static class Sub extends Base {
        public String getName() {
            return "Sub";
        }
    }
}