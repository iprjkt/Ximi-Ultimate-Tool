import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.PrintStream;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.List;

/**
 * Prints every package known to the device with the label the launcher shows.
 *
 * `pm` cannot print labels and phones ship without aapt, so this runs inside
 * app_process and asks PackageManager directly. Only reflection is used, so it
 * compiles with a plain JDK (no android.jar needed).
 *
 *   CLASSPATH=/data/local/tmp/ximi-pkglabels.jar app_process /system/bin XimiPkgLabels
 *
 * Output, one line per package (tab separated, UTF-8):
 *   package  system(0|1)  enabled(0|1)  installed(0|1)  label
 */
public final class XimiPkgLabels {
    private static final int FLAG_SYSTEM = 0x00000001;
    private static final int FLAG_INSTALLED = 0x00800000;
    // Includes packages removed with `pm uninstall -k --user 0`, so restore lists get names too.
    private static final int MATCH_UNINSTALLED_PACKAGES = 0x00002000;

    public static void main(String[] args) throws Exception {
        PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out), false, "UTF-8");
        Object pm = packageManager();
        Method getApps = pm.getClass().getMethod("getInstalledApplications", int.class);
        Class<?> appInfoClass = Class.forName("android.content.pm.ApplicationInfo");
        Method getLabel = pm.getClass().getMethod("getApplicationLabel", appInfoClass);
        Field pkgField = appInfoClass.getField("packageName");
        Field flagsField = appInfoClass.getField("flags");
        Field enabledField = appInfoClass.getField("enabled");

        List<?> apps = (List<?>) getApps.invoke(pm, MATCH_UNINSTALLED_PACKAGES);
        for (Object ai : apps) {
            String pkg = (String) pkgField.get(ai);
            int flags = flagsField.getInt(ai);
            boolean enabled = enabledField.getBoolean(ai);
            String label;
            try {
                Object cs = getLabel.invoke(pm, ai);
                label = cs == null ? "" : clean(cs.toString());
            } catch (Throwable t) {
                // A broken resource table in one app must not stop the whole listing.
                label = "";
            }
            out.print(pkg);
            out.print('\t');
            out.print((flags & FLAG_SYSTEM) != 0 ? '1' : '0');
            out.print('\t');
            out.print(enabled ? '1' : '0');
            out.print('\t');
            out.print((flags & FLAG_INSTALLED) != 0 ? '1' : '0');
            out.print('\t');
            out.print(label);
            out.print('\n');
        }
        out.flush();
        // Skip ART shutdown hooks that occasionally hang under app_process.
        System.exit(0);
    }

    private static String clean(String s) {
        StringBuilder b = new StringBuilder(s.length());
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            b.append(Character.isISOControl(c) ? ' ' : c);
        }
        return b.toString().trim();
    }

    /** Same bootstrap scrcpy uses: a bare ActivityThread is enough to get the system context. */
    private static Object packageManager() throws Exception {
        Class<?> looper = Class.forName("android.os.Looper");
        if (looper.getMethod("getMainLooper").invoke(null) == null) {
            looper.getMethod("prepareMainLooper").invoke(null);
        }
        Class<?> atClass = Class.forName("android.app.ActivityThread");
        Constructor<?> ctor = atClass.getDeclaredConstructor();
        ctor.setAccessible(true);
        Object thread = ctor.newInstance();
        Field current = atClass.getDeclaredField("sCurrentActivityThread");
        current.setAccessible(true);
        current.set(null, thread);
        Method getSystemContext = atClass.getDeclaredMethod("getSystemContext");
        getSystemContext.setAccessible(true);
        Object context = getSystemContext.invoke(thread);
        return context.getClass().getMethod("getPackageManager").invoke(context);
    }
}
