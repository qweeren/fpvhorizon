/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone;

import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

public final class DroneRegistry {
    private static final Set<UUID> ACTIVE = ConcurrentHashMap.newKeySet();

    private DroneRegistry() {
    }

    public static boolean isDrone(UUID id) {
        return ACTIVE.contains(id);
    }

    public static void set(UUID id, boolean on) {
        if (on) {
            ACTIVE.add(id);
        } else {
            ACTIVE.remove(id);
        }
    }

    public static Set<UUID> snapshot() {
        return Set.copyOf(ACTIVE);
    }

    public static void clear() {
        ACTIVE.clear();
    }
}
