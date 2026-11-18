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

public final class PhysicsSettings {
    private static final float[] values = new float[Param.values().length];
    private static boolean loaded = false;
    private static int version = 0;

    private PhysicsSettings() {
    }

    public static float get(Param p) {
        PhysicsSettings.ensureLoaded();
        return values[p.ordinal()];
    }

    public static void set(Param p, float value) {
        PhysicsSettings.ensureLoaded();
        PhysicsSettings.values[p.ordinal()] = PhysicsSettings.snap(p, value);
        ++version;
    }

    public static void reset() {
        PhysicsSettings.ensureLoaded();
        for (Param p : Param.values()) {
            PhysicsSettings.values[p.ordinal()] = p.defaultValue;
        }
        ++version;
        PhysicsSettings.save();
    }

    public static int getVersion() {
        PhysicsSettings.ensureLoaded();
        return version;
    }

    private static float snap(Param p, float value) {
        float steps = Math.round((value - p.min) / p.step);
        float snapped = p.min + steps * p.step;
        return Math.max(p.min, Math.min(p.max, snapped));
    }

    private static Path configPath() {
        return FMLPaths.CONFIGDIR.get().resolve("fpvdrone-physics.properties");
    }

    private static void ensureLoaded() {
        if (loaded) {
            return;
        }
        loaded = true;
        Path path = PhysicsSettings.configPath();
        if (!Files.exists(path, new LinkOption[0])) {
            return;
        }
        Properties props = new Properties();
        try (InputStream in = Files.newInputStream(path, new OpenOption[0]);){
            props.load(in);
        }
        catch (IOException e) {
            FpvDroneMod.LOGGER.warn("[fpvdrone] could not read physics config", (Throwable)e);
            return;
        }
        for (Param p : Param.values()) {
            try {
                float v = Float.parseFloat(props.getProperty(p.key, String.valueOf(p.defaultValue)).trim());
                PhysicsSettings.values[p.ordinal()] = Math.max(p.min, Math.min(p.max, v));
            }
            catch (NumberFormatException ignored) {
                PhysicsSettings.values[p.ordinal()] = p.defaultValue;
            }
        }
        ++version;
    }

    public static void save() {
        PhysicsSettings.ensureLoaded();
        Properties props = new Properties();
        for (Param p : Param.values()) {
            props.setProperty(p.key, String.valueOf(values[p.ordinal()]));
        }
        try (OutputStream out = Files.newOutputStream(PhysicsSettings.configPath(), new OpenOption[0]);){
            props.store(out, "FPV Drone physics constants, in SI units.");
        }
        catch (IOException e) {
            FpvDroneMod.LOGGER.warn("[fpvdrone] could not write physics config", (Throwable)e);
        }
    }

    static {
        for (Param p : Param.values()) {
            PhysicsSettings.values[p.ordinal()] = p.defaultValue;
        }
    }

    public static enum Param {
        AIR_DENSITY("air_density", 1.225f, 0.1f, 3.0f, 0.005f, 1.0f, 3, " kg/m\u00b3"),
        DRONE_DENSITY("drone_density", 1500.0f, 200.0f, 5000.0f, 50.0f, 1.0f, 0, " kg/m\u00b3"),
        DRAG_COEFFICIENT("drag_coefficient", 1.05f, 0.1f, 3.0f, 0.01f, 1.0f, 2, ""),
        RESISTANCE_PER_1000_FT("resistance_per_1000_ft", 25.7f, 1.0f, 100.0f, 0.1f, 1.0f, 1, " ohm/kft"),
        TEMP_COEFF("temp_coeff", 0.00393f, 0.0f, 0.01f, 1.0E-5f, 1000.0f, 2, "e-3/K"),
        SPECIFIC_HEAT("specific_heat", 400.0f, 100.0f, 1000.0f, 5.0f, 1.0f, 0, " J/kgK"),
        HEAT_TRANSFER("heat_transfer", 50.0f, 1.0f, 500.0f, 1.0f, 1.0f, 0, " W/m2K");

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
