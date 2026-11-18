/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 *  net.minecraft.server.MinecraftServer
 *  net.minecraft.server.level.ServerLevel
 *  net.minecraft.world.level.storage.LevelData$RespawnData
 */
package com.taylorgoolsby.fpvdrone.race;

import net.minecraft.core.BlockPos;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.storage.LevelData;

public final class RaceSpawn {
    private RaceSpawn() {
    }

    public static Spawn get(MinecraftServer server) {
        LevelData.RespawnData data = server.overworld().getRespawnData();
        ServerLevel level = server.getLevel(data.dimension());
        if (level == null) {
            return null;
        }
        return new Spawn(level, data.pos(), data.yaw());
    }

    public record Spawn(ServerLevel level, BlockPos pos, float yaw) {
    }
}
