/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 *  net.minecraft.server.level.ServerLevel
 */
package com.taylorgoolsby.fpvdrone.race;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;

public record RaceSpawn.Spawn(ServerLevel level, BlockPos pos, float yaw) {
}
