/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraftforge.fml.loading.FMLPaths
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.FpvDroneMod;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.OpenOption;
import java.nio.file.Path;
import java.util.Locale;
import java.util.Properties;
import net.minecraftforge.fml.loading.FMLPaths;

public final class DroneBuild {
    public static final float MOTOR_DENSITY = 13240.0f;
    public static final float BELL_MASS_FRACTION = 0.36363637f;
    public static final float BLADE_DENSITY = 1200.0f;
    public static final float BLADE_THICKNESS = 0.0013f;
    private static final float[] values = new float[Param.values().length];
    private static boolean loaded = false;
    private static int version = 0;

    private DroneBuild() {
    }

    public static float get(Param p) {
        DroneBuild.ensureLoaded();
        return values[p.ordinal()];
    }

    public static void set(Param p, float value) {
        DroneBuild.ensureLoaded();
        DroneBuild.values[p.ordinal()] = DroneBuild.snap(p, value);
        ++version;
    }

    public static void reset() {
        DroneBuild.ensureLoaded();
        for (Param p : Param.values()) {
            DroneBuild.values[p.ordinal()] = p.defaultValue;
        }
        ++version;
        DroneBuild.save();
    }

    public static int getVersion() {
        DroneBuild.ensureLoaded();
        return version;
    }

    public static float motorMass() {
        float radius = DroneBuild.get(Param.MOTOR_WIDTH) * 0.5f;
        float volume = (float)Math.PI * radius * radius * DroneBuild.get(Param.MOTOR_HEIGHT);
        return 13240.0f * volume;
    }

    public static float bellMass() {
        return 0.36363637f * DroneBuild.motorMass();
    }

    public static float bladeMass() {
        float volume = DroneBuild.get(Param.PROP_RADIUS) * DroneBuild.get(Param.PROP_WIDTH) * 0.0013f;
        return 1200.0f * volume;
    }

    private static float snap(Param p, float value) {
        float steps = Math.round((value - p.min) / p.step);
        float snapped = p.min + steps * p.step;
        return Math.max(p.min, Math.min(p.max, snapped));
    }

    private static Path configPath() {
        return FMLPaths.CONFIGDIR.get().resolve("fpvdrone-build.properties");
    }

    private static void ensureLoaded() {
        if (loaded) {
            return;
        }
        loaded = true;
        Path path = DroneBuild.configPath();
        if (!Files.exists(path, new LinkOption[0])) {
            return;
        }
        Properties props = new Properties();
        try (InputStream in = Files.newInputStream(path, new OpenOption[0]);){
            props.load(in);
        }
        catch (IOException e) {
            FpvDroneMod.LOGGER.warn("[fpvdrone] could not read build config", (Throwable)e);
            return;
        }
        for (Param p : Param.values()) {
            try {
                float v = Float.parseFloat(props.getProperty(p.key, String.valueOf(p.defaultValue)).trim());
                DroneBuild.values[p.ordinal()] = Math.max(p.min, Math.min(p.max, v));
            }
            catch (NumberFormatException ignored) {
                DroneBuild.values[p.ordinal()] = p.defaultValue;
            }
        }
        ++version;
    }

    public static void save() {
        DroneBuild.ensureLoaded();
        Properties props = new Properties();
        for (Param p : Param.values()) {
            props.setProperty(p.key, String.valueOf(values[p.ordinal()]));
        }
        try (OutputStream out = Files.newOutputStream(DroneBuild.configPath(), new OpenOption[0]);){
            props.store(out, "FPV Drone build. SI units: kg for masses, m for lengths, rpm per volt for kv.");
        }
        catch (IOException e) {
            FpvDroneMod.LOGGER.warn("[fpvdrone] could not write build config", (Throwable)e);
        }
    }

    static {
        for (Param p : Param.values()) {
            DroneBuild.values[p.ordinal()] = p.defaultValue;
        }
    }

    public static enum Param {
        DRONE_MASS("drone_mass", 0.65f, 0.1f, 2.0f, 0.005f, 1000.0f, 0, " g"),
        BATTERY_CELLS("battery_cells", 2.0f, 1.0f, 8.0f, 1.0f, 1.0f, 0, "S"),
        MOTOR_KV("motor_kv", 2000.0f, 500.0f, 5000.0f, 50.0f, 1.0f, 0, " kV"),
        BLADES("blades", 3.0f, 2.0f, 6.0f, 1.0f, 1.0f, 0, ""),
        PROP_RADIUS("prop_radius", 0.0635f, 0.025f, 0.15f, 5.0E-4f, 1000.0f, 1, " mm"),
        PROP_PITCH("prop_pitch", 0.1f, 0.03f, 0.25f, 0.001f, 1000.0f, 0, " mm"),
        PROP_WIDTH("prop_width", 0.015f, 0.005f, 0.04f, 5.0E-4f, 1000.0f, 1, " mm"),
        MOTOR_WIDTH("motor_width", 0.023f, 0.01f, 0.06f, 5.0E-4f, 1000.0f, 1, " mm"),
        MOTOR_HEIGHT("motor_height", 0.006f, 0.003f, 0.03f, 5.0E-4f, 1000.0f, 1, " mm"),
        CAMERA_ANGLE("camera_angle", 10.0f, 0.0f, 70.0f, 1.0f, 1.0f, 0, "\u00b0");

        public final String key;
        public final float defaultValue;
        public final float min;
        public final float max;
        public final float step;
        public final float displayScale;
        public final int decimals;
        public final String unit;

        private Param(String key, float defaultValue, float min, float max, float step, float displayScale, int decimals, String unit) {
            this.key = key;
            this.defaultValue = defaultValue;
            this.min = min;
            this.max = max;
            this.step = step;
            this.displayScale = displayScale;
            this.decimals = decimals;
            this.unit = unit;
        }

        public String format(float value) {
            return String.format(Locale.ROOT, "%." + this.decimals + "f%s", Float.valueOf(value * this.displayScale), this.unit);
        }
    }
}
