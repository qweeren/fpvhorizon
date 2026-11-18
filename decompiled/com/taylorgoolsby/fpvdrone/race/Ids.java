/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race;

import java.util.UUID;

public final class Ids {
    private Ids() {
    }

    public static String newId() {
        return Ids.toHex(UUID.randomUUID());
    }

    public static String toHex(UUID uuid) {
        return uuid.toString().replace("-", "").toUpperCase();
    }

    public static UUID fromHex(String hex) {
        String h = hex.replace("-", "");
        return new UUID(Long.parseUnsignedLong(h.substring(0, 16), 16), Long.parseUnsignedLong(h.substring(16), 16));
    }

    public static String sanitizeName(String value) {
        String v = value.replaceAll("[^!-~]", "").replace("\\", "");
        if (v.length() > 32) {
            v = v.substring(0, 32);
        }
        return v;
    }
}
