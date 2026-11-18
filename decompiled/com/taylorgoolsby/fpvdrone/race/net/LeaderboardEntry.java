/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race.net;

import java.util.UUID;

public record LeaderboardEntry(UUID userId, String name, int millis) {
}
