/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  org.joml.Quaternionf
 *  org.joml.Quaternionfc
 */
package com.taylorgoolsby.fpvdrone;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;

public final class DroneOrientations {
    private static final float SMOOTHING_SECONDS = 0.04f;
    private static final Map<UUID, Entry> ENTRIES = new ConcurrentHashMap<UUID, Entry>();

    private DroneOrientations() {
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    public static void set(UUID id, float x, float y, float z, float w) {
        Entry entry;
        float lengthSquared = x * x + y * y + z * z + w * w;
        if (!Float.isFinite(lengthSquared) || lengthSquared < 1.0E-6f) {
            return;
        }
        float inverseLength = 1.0f / (float)Math.sqrt(lengthSquared);
        float nx = x * inverseLength;
        float ny = y * inverseLength;
        float nz = z * inverseLength;
        float nw = w * inverseLength;
        Entry entry2 = entry = ENTRIES.computeIfAbsent(id, key -> {
            Entry created = new Entry();
            created.display.set(nx, ny, nz, nw);
            created.lastNanos = System.nanoTime();
            return created;
        });
        synchronized (entry2) {
            entry.target.set(nx, ny, nz, nw);
        }
    }

    public static boolean has(UUID id) {
        return ENTRIES.containsKey(id);
    }

    /*
     * WARNING - Removed try catching itself - possible behaviour change.
     */
    public static boolean get(UUID id, Quaternionf dest) {
        Entry entry = ENTRIES.get(id);
        if (entry == null) {
            return false;
        }
        Entry entry2 = entry;
        synchronized (entry2) {
            long now = System.nanoTime();
            float dt = (float)(now - entry.lastNanos) * 1.0E-9f;
            entry.lastNanos = now;
            float alpha = 1.0f - (float)Math.exp(-dt / 0.04f);
            entry.display.slerp((Quaternionfc)entry.target, alpha);
            dest.set((Quaternionfc)entry.display);
        }
        return true;
    }

    public static void remove(UUID id) {
        ENTRIES.remove(id);
    }

    public static void clear() {
        ENTRIES.clear();
    }

    private static final class Entry {
        final Quaternionf target = new Quaternionf();
        final Quaternionf display = new Quaternionf();
        long lastNanos;

        private Entry() {
        }
    }
}
