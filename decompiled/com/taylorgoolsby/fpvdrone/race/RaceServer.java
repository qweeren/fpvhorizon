/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.ChatFormatting
 *  net.minecraft.core.BlockPos
 *  net.minecraft.network.chat.Component
 *  net.minecraft.server.MinecraftServer
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraft.world.level.Level
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.DroneRegistry;
import com.taylorgoolsby.fpvdrone.race.Ids;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.function.BiPredicate;
import java.util.function.Consumer;
import net.minecraft.ChatFormatting;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.Level;

public final class RaceServer {
    public static final String VERSION = "26.3";
    public static volatile MinecraftServer server;
    public static volatile boolean databaseReady;
    public static volatile String databaseError;

    private RaceServer() {
    }

    public static void databaseUnavailable(ServerPlayer player) {
        RaceServer.error(player, "Racing is unavailable because the race database could not be opened. " + databaseError);
    }

    public static <T> void then(CompletableFuture<T> future, Consumer<T> onSuccess, Consumer<Throwable> onError) {
        future.whenComplete((value, error) -> {
            MinecraftServer s = server;
            if (s == null) {
                return;
            }
            s.execute(() -> {
                if (error != null) {
                    Throwable cause = error instanceof CompletionException && error.getCause() != null ? error.getCause() : error;
                    onError.accept(cause);
                } else {
                    onSuccess.accept(value);
                }
            });
        });
    }

    public static String getId(UUID uuid) {
        return Ids.toHex(uuid);
    }

    public static ServerPlayer getPlayer(String userId) {
        MinecraftServer s = server;
        if (s == null) {
            return null;
        }
        return s.getPlayerList().getPlayer(Ids.fromHex(userId));
    }

    public static String dimensionOf(Level level) {
        return level.dimension().identifier().toString();
    }

    public static BiPredicate<String, BlockPos> solidChecker(Level level) {
        String dimension = RaceServer.dimensionOf(level);
        return (testDimension, testPos) -> testDimension.equals(dimension) && level.getBlockState(testPos).isSolid();
    }

    public static boolean isFlying(UUID uuid) {
        return DroneRegistry.isDrone(uuid);
    }

    public static void info(ServerPlayer player, String message) {
        RaceServer.send(player, message, ChatFormatting.YELLOW);
    }

    public static void error(ServerPlayer player, String message) {
        if (message != null && message.startsWith("java.lang.Exception: ")) {
            message = message.substring(21);
        }
        RaceServer.send(player, message, ChatFormatting.RED);
    }

    public static void success(ServerPlayer player, String message) {
        RaceServer.send(player, message, ChatFormatting.GREEN);
    }

    private static void send(ServerPlayer player, String message, ChatFormatting color) {
        if (player == null || message == null) {
            return;
        }
        player.sendSystemMessage((Component)Component.literal((String)message).withStyle(color));
    }

    static {
        databaseReady = false;
        databaseError = "";
    }
}
