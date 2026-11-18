/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.FpvDroneMod;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

public final class DroneDebug {
    private static volatile boolean enabled = Boolean.getBoolean("fpvdrone.debug");
    private static final long LOG_INTERVAL_MILLIS = 1000L;
    private static final long STALE_NANOS = 2000000000L;
    private static final long RATE_WINDOW_NANOS = 500000000L;
    private static final Map<String, Long> LAST_LOG = new ConcurrentHashMap<String, Long>();
    private static final Map<UUID, Head> HEADS = new ConcurrentHashMap<UUID, Head>();
    private static volatile long hookNanos;
    private static volatile boolean hookDrone;

    private DroneDebug() {
    }

    public static boolean isEnabled() {
        return enabled;
    }

    public static void setEnabled(boolean value) {
        enabled = value;
        if (!value) {
            HEADS.clear();
            hookNanos = 0L;
        }
    }

    public static void log(String key, String message) {
        if (!enabled) {
            return;
        }
        long now = System.currentTimeMillis();
        Long last = LAST_LOG.get(key);
        if (last != null && now - last < 1000L) {
            return;
        }
        LAST_LOG.put(key, now);
        FpvDroneMod.LOGGER.info("[fpvdrone-debug] {}", (Object)message);
    }

    public static void markHook(boolean drone) {
        if (!enabled) {
            return;
        }
        hookNanos = System.nanoTime();
        hookDrone = drone;
    }

    public static void markRender(UUID id, String name, boolean local, boolean hasOrientation, float stateBoxHeight) {
        if (!enabled) {
            return;
        }
        Head head = HEADS.computeIfAbsent(id, key -> new Head());
        head.name = name;
        head.local = local;
        head.hasOrientation = hasOrientation;
        head.stateBoxHeight = stateBoxHeight;
        head.renderNanos = System.nanoTime();
    }

    public static void markReceived(UUID id) {
        if (!enabled) {
            return;
        }
        Head head = HEADS.computeIfAbsent(id, key -> new Head());
        ++head.receivedCount;
        head.receivedNanos = System.nanoTime();
    }

    public static void markApplied(UUID id, float yaw, float pitch, float roll) {
        if (!enabled) {
            return;
        }
        Head head = HEADS.computeIfAbsent(id, key -> new Head());
        head.yaw = yaw;
        head.pitch = pitch;
        head.roll = roll;
        head.appliedNanos = System.nanoTime();
    }

    public static String screenText() {
        long now = System.nanoTime();
        StringBuilder sb = new StringBuilder("Head sync: mixin ");
        if (hookNanos == 0L) {
            sb.append("NOT RUNNING");
        } else {
            sb.append("ok ").append(DroneDebug.millis(now - hookNanos)).append("ms");
        }
        boolean any = false;
        for (Head head : HEADS.values()) {
            long window;
            long latest = Math.max(head.renderNanos, head.receivedNanos);
            if (now - latest > 2000000000L) continue;
            any = true;
            if (head.rateNanos == 0L) {
                head.rateNanos = now;
                head.rateCount = head.receivedCount;
            }
            if ((window = now - head.rateNanos) > 500000000L) {
                head.ratePerSecond = (float)(head.receivedCount - head.rateCount) * 1.0E9f / (float)window;
                head.rateCount = head.receivedCount;
                head.rateNanos = now;
            }
            sb.append(" | ").append(head.name);
            if (head.local) {
                sb.append(" (you)");
            }
            sb.append(head.hasOrientation ? " orient yes" : " orient NO");
            sb.append(String.format(Locale.ROOT, " rx %.0f/s", Float.valueOf(head.ratePerSecond)));
            if (head.receivedNanos != 0L) {
                sb.append(" last ").append(DroneDebug.millis(now - head.receivedNanos)).append("ms");
            } else {
                sb.append(" never received");
            }
            if (now - head.renderNanos > 2000000000L) {
                sb.append(" NOT DRAWN");
            } else if (now - head.appliedNanos < 2000000000L) {
                sb.append(String.format(Locale.ROOT, " ypr %.0f %.0f %.0f", Float.valueOf(head.yaw), Float.valueOf(head.pitch), Float.valueOf(head.roll)));
            } else {
                sb.append(" rotation NOT APPLIED");
            }
            sb.append(String.format(Locale.ROOT, " box %.2f", Float.valueOf(head.stateBoxHeight)));
        }
        if (!any) {
            sb.append(" | no drone players seen");
        }
        return sb.toString();
    }

    private static long millis(long nanos) {
        return nanos / 1000000L;
    }

    private static final class Head {
        String name = "?";
        boolean local;
        boolean hasOrientation;
        float stateBoxHeight;
        long renderNanos;
        long receivedCount;
        long receivedNanos;
        long rateCount;
        long rateNanos;
        float ratePerSecond;
        long appliedNanos;
        float yaw;
        float pitch;
        float roll;

        private Head() {
        }
    }
}
