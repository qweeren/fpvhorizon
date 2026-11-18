/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.brigadier.CommandDispatcher
 *  com.mojang.brigadier.Message
 *  com.mojang.brigadier.arguments.ArgumentType
 *  com.mojang.brigadier.arguments.StringArgumentType
 *  com.mojang.brigadier.builder.LiteralArgumentBuilder
 *  com.mojang.brigadier.context.CommandContext
 *  com.mojang.brigadier.exceptions.CommandSyntaxException
 *  com.mojang.brigadier.exceptions.SimpleCommandExceptionType
 *  com.mojang.brigadier.suggestion.SuggestionProvider
 *  net.minecraft.commands.CommandSourceStack
 *  net.minecraft.commands.Commands
 *  net.minecraft.network.chat.Component
 *  net.minecraft.server.MinecraftServer
 *  net.minecraft.server.level.ServerLevel
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraft.server.permissions.PermissionCheck
 *  net.minecraft.world.level.Level
 */
package com.taylorgoolsby.fpvdrone.race;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.Message;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.brigadier.exceptions.SimpleCommandExceptionType;
import com.mojang.brigadier.suggestion.SuggestionProvider;
import com.taylorgoolsby.fpvdrone.race.BlockKey;
import com.taylorgoolsby.fpvdrone.race.RaceBuild;
import com.taylorgoolsby.fpvdrone.race.RaceNavigate;
import com.taylorgoolsby.fpvdrone.race.RaceServer;
import com.taylorgoolsby.fpvdrone.race.RaceSpawn;
import com.taylorgoolsby.fpvdrone.race.db.RaceRepository;
import java.util.Set;
import java.util.function.Predicate;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.permissions.PermissionCheck;
import net.minecraft.world.level.Level;

public final class RaceCommand {
    private static final SuggestionProvider<CommandSourceStack> TRACK_NAMES = (context, builder) -> {
        if (!RaceServer.databaseReady) {
            return builder.buildFuture();
        }
        return RaceRepository.listTrackNames().thenApply(names -> {
            for (String name : names) {
                builder.suggest(name);
            }
            return builder.build();
        });
    };

    private RaceCommand() {
    }

    public static void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        LiteralArgumentBuilder race = Commands.literal((String)"race");
        race.executes(RaceCommand::help);
        race.then(Commands.literal((String)"setName").then(Commands.argument((String)"name", (ArgumentType)StringArgumentType.string()).executes(RaceCommand::setName)));
        race.then(Commands.literal((String)"list").executes(RaceCommand::list));
        race.then(Commands.literal((String)"tp").then(Commands.argument((String)"name", (ArgumentType)StringArgumentType.string()).suggests(TRACK_NAMES).executes(RaceCommand::teleport)));
        race.then(Commands.literal((String)"spawn").executes(RaceCommand::spawn));
        race.then(((LiteralArgumentBuilder)Commands.literal((String)"delete").requires((Predicate)Commands.hasPermission((PermissionCheck)Commands.LEVEL_GAMEMASTERS))).then(Commands.argument((String)"name", (ArgumentType)StringArgumentType.string()).suggests(TRACK_NAMES).executes(RaceCommand::delete)));
        dispatcher.register(race);
    }

    private static int help(CommandContext<CommandSourceStack> context) {
        ((CommandSourceStack)context.getSource()).sendSuccess(() -> Component.literal((String)"Manage race tracks with this command.\n/race setName <name> - Give a name to the track you are currently building.\n/race list - List the names of all race tracks.\n/race tp <name> - Teleport to a race track.\n/race spawn - Teleport to the world spawn.\n/race delete <name> - Delete a race track (operators only)."), false);
        return 1;
    }

    private static int setName(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {
        ServerPlayer caller = ((CommandSourceStack)context.getSource()).getPlayerOrException();
        String userId = RaceServer.getId(caller.getUUID());
        if (!RaceBuild.checkBuildingMode(userId)) {
            throw new SimpleCommandExceptionType((Message)Component.literal((String)"You are not in track building mode.")).create();
        }
        String name = StringArgumentType.getString(context, (String)"name");
        RaceBuild.setTrackName(userId, name, caller);
        return 1;
    }

    private static int list(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {
        RaceCommand.requireDatabase();
        CommandSourceStack source = (CommandSourceStack)context.getSource();
        RaceServer.then(RaceRepository.listTrackNames(), names -> {
            if (names.isEmpty()) {
                source.sendSuccess(() -> Component.literal((String)"There are no race tracks yet."), false);
            } else {
                source.sendSuccess(() -> Component.literal((String)("Race tracks (" + names.size() + "): " + String.join((CharSequence)", ", names))), false);
            }
        }, error -> source.sendFailure((Component)Component.literal((String)("Unable to list the tracks. " + error.getMessage()))));
        return 1;
    }

    private static int delete(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {
        RaceCommand.requireDatabase();
        String name = StringArgumentType.getString(context, (String)"name");
        CommandSourceStack source = (CommandSourceStack)context.getSource();
        String who = source.getTextName();
        RaceServer.then(RaceRepository.softDeleteTrackByName(name), track -> {
            if (track == null) {
                source.sendFailure((Component)Component.literal((String)("There is no race track named '" + name + "'. Use /race list to see them.")));
                return;
            }
            RaceNavigate.exitAllPlayersFromTrack(new BlockKey(track.dimension, track.getStartPos()), "The track was deleted by " + who);
            source.sendSuccess(() -> Component.literal((String)("Track '" + track.name + "' was deleted.")), true);
        }, error -> source.sendFailure((Component)Component.literal((String)("Unable to delete the track. " + error.getMessage()))));
        return 1;
    }

    private static int teleport(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {
        ServerPlayer player = ((CommandSourceStack)context.getSource()).getPlayerOrException();
        RaceCommand.requireDatabase();
        String name = StringArgumentType.getString(context, (String)"name");
        CommandSourceStack source = (CommandSourceStack)context.getSource();
        RaceServer.then(RaceRepository.getTrackByName(name), track -> {
            if (track == null) {
                source.sendFailure((Component)Component.literal((String)("There is no race track named '" + name + "'. Use /race list to see them.")));
                return;
            }
            MinecraftServer server = source.getServer();
            ServerLevel destination = null;
            for (ServerLevel level : server.getAllLevels()) {
                if (!RaceServer.dimensionOf((Level)level).equals(track.dimension)) continue;
                destination = level;
                break;
            }
            if (destination == null) {
                source.sendFailure((Component)Component.literal((String)("The dimension of track '" + track.name + "' (" + track.dimension + ") is not available.")));
                return;
            }
            RaceCommand.teleportPlayer(player, destination, (double)track.startPosX + 0.5, (double)track.startPosY + 1.0, (double)track.startPosZ + 0.5, player.getYRot());
            RaceServer.success(player, "Teleported to track '" + track.name + "'.");
        }, error -> source.sendFailure((Component)Component.literal((String)("Unable to look up the track. " + error.getMessage()))));
        return 1;
    }

    private static int spawn(CommandContext<CommandSourceStack> context) throws CommandSyntaxException {
        ServerPlayer player = ((CommandSourceStack)context.getSource()).getPlayerOrException();
        RaceSpawn.Spawn spawn = RaceSpawn.get(((CommandSourceStack)context.getSource()).getServer());
        if (spawn == null) {
            throw new SimpleCommandExceptionType((Message)Component.literal((String)"The world spawn's dimension is not available.")).create();
        }
        RaceCommand.teleportPlayer(player, spawn.level(), (double)spawn.pos().getX() + 0.5, spawn.pos().getY(), (double)spawn.pos().getZ() + 0.5, spawn.yaw());
        RaceServer.success(player, "Teleported to spawn.");
        return 1;
    }

    private static void teleportPlayer(ServerPlayer player, ServerLevel level, double x, double y, double z, float yaw) {
        RaceNavigate.exitRacingMode(RaceServer.getId(player.getUUID()), player, null);
        player.teleportTo(level, x, y, z, Set.of(), yaw, player.getXRot(), true);
    }

    private static void requireDatabase() throws CommandSyntaxException {
        if (!RaceServer.databaseReady) {
            throw new SimpleCommandExceptionType((Message)Component.literal((String)("Racing is unavailable because the race database could not be opened. " + RaceServer.databaseError))).create();
        }
    }
}
