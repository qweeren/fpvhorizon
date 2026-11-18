/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race;

public final class RaceTime {
    private RaceTime() {
    }

    public static String format(long millis) {
        if (millis < 0L) {
            millis = 0L;
        }
        long minutes = millis / 60000L;
        long seconds = millis / 1000L % 60L;
        long centis = millis % 1000L / 10L;
        return String.format("%d:%02d.%02d", minutes, seconds, centis);
    }
}
