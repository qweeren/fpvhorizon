/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 *  net.minecraft.core.Direction
 *  net.minecraft.server.level.ServerPlayer
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.BlockKey;
import com.taylorgoolsby.fpvdrone.race.Ids;
import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.RaceGateException;
import com.taylorgoolsby.fpvdrone.race.RaceMath;
import com.taylorgoolsby.fpvdrone.race.RaceNavigate;
import com.taylorgoolsby.fpvdrone.race.RaceNet;
import com.taylorgoolsby.fpvdrone.race.RaceServer;
import com.taylorgoolsby.fpvdrone.race.db.RaceRepository;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;
import java.util.function.BiPredicate;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.level.ServerPlayer;

public final class RaceBuild {
    public static final int MAX_GATES_PER_TRACK = 150;
    private static final Map<String, BlockKey> startingBlock = new HashMap<String, BlockKey>();
    private static final Map<String, RaceGate[]> gateChoices = new HashMap<String, RaceGate[]>();
    private static final Map<String, ArrayList<RaceGate>> completedGates = new HashMap<String, ArrayList<RaceGate>>();
    private static final Map<String, String> trackName = new HashMap<String, String>();

    private RaceBuild() {
    }

    public static boolean checkBuildingMode(String userId) {
        return startingBlock.containsKey(userId);
    }

    public static void startTrack(String userId, String dimension, BlockPos pos, ServerPlayer player) {
        if (!RaceServer.databaseReady) {
            RaceServer.databaseUnavailable(player);
            return;
        }
        if (RaceBuild.checkBuildingMode(userId)) {
            RaceServer.error(player, "You are already building a track. Break its target block to start over.");
            return;
        }
        startingBlock.put(userId, new BlockKey(dimension, pos));
        gateChoices.remove(userId);
        completedGates.put(userId, new ArrayList());
        trackName.put(userId, "");
        RaceServer.info(player, "You are now building a track. Right click gates with a stick to add them, in the order they are to be flown.");
        RaceNet.sendBuildMode(player, true);
    }

    public static void exitBuildingMode(String userId, ServerPlayer player) {
        boolean wasBuilding = startingBlock.remove(userId) != null;
        gateChoices.remove(userId);
        completedGates.remove(userId);
        trackName.remove(userId);
        if (wasBuilding && player != null) {
            RaceServer.info(player, "You are no longer building a track.");
            RaceNet.sendBuildMode(player, false);
        }
    }

    public static void setTrackName(String userId, String name, ServerPlayer player) {
        if (name.isEmpty()) {
            RaceServer.error(player, "Track name cannot be blank.");
            return;
        }
        String sanitizedName = Ids.sanitizeName(name);
        if (!sanitizedName.equals(name)) {
            RaceServer.error(player, "Track name can only contain standard keyboard characters, at most 32 of them. Space and backslash are not allowed.");
            return;
        }
        RaceServer.then(RaceRepository.getTrackByName(sanitizedName), existing -> {
            if (!RaceBuild.checkBuildingMode(userId)) {
                return;
            }
            if (existing != null) {
                RaceServer.error(player, "That name is already taken.");
            } else {
                trackName.put(userId, sanitizedName);
                RaceServer.success(player, "Track name set to '" + sanitizedName + "'.");
            }
        }, error -> RaceServer.error(player, "Unable to check the track name: " + error.getMessage()));
    }

    public static void addBlock(String userId, String dimension, BlockPos pos, Direction testFace, BiPredicate<String, BlockPos> checkSolid, ServerPlayer player) {
        if (testFace == null) {
            return;
        }
        if (!RaceBuild.checkBuildingMode(userId)) {
            return;
        }
        BlockKey blockKey = new BlockKey(dimension, pos);
        ArrayList<RaceGate> currentCompletedGates = completedGates.get(userId);
        BlockKey startKey = startingBlock.get(userId);
        if (startKey.equals(blockKey)) {
            RaceBuild.completeTrack(userId, player);
            return;
        }
        if (!dimension.equals(startKey.dimension)) {
            RaceServer.error(player, "Gates must be in the same dimension as the target block.");
            return;
        }
        if (currentCompletedGates.size() >= 150) {
            RaceServer.error(player, "The max number of gates has been reached (150).");
            return;
        }
        if (!checkSolid.test(dimension, pos)) {
            RaceServer.error(player, "Gate block must be solid.");
            return;
        }
        RaceGate[] storedChoices = gateChoices.get(userId);
        if (storedChoices != null) {
            boolean found1 = storedChoices[0].path.contains(pos);
            boolean found2 = storedChoices[1].path.contains(pos);
            gateChoices.remove(userId);
            if (found1 == found2) {
                RaceServer.error(player, "The block you chose does not specify which gate you want to add.");
                return;
            }
            RaceBuild.addGate(currentCompletedGates, found1 ? storedChoices[0] : storedChoices[1], player);
            return;
        }
        ArrayList<BlockPos> neighbors = RaceBuild.findSolidNeighbors(pos, testFace, dimension, checkSolid);
        if (neighbors.isEmpty()) {
            RaceServer.error(player, "Either you selected the wrong face, or this is not a gate.");
            return;
        }
        if (neighbors.size() == 1) {
            try {
                RaceGate gate = RaceMath.loadGate(dimension, pos, testFace, neighbors.get(0), checkSolid, null);
                RaceBuild.addGate(currentCompletedGates, gate, player);
            }
            catch (RaceGateException e) {
                RaceServer.error(player, "Failed to add a gate. " + e.endUserReason);
            }
            return;
        }
        RaceGate gate1 = null;
        RaceGate gate2 = null;
        RaceGateException e1 = null;
        RaceGateException e2 = null;
        try {
            gate1 = RaceMath.loadGate(dimension, pos, testFace, neighbors.get(0), checkSolid, null);
        }
        catch (RaceGateException e) {
            e1 = e;
        }
        try {
            gate2 = RaceMath.loadGate(dimension, pos, testFace, neighbors.get(1), checkSolid, null);
        }
        catch (RaceGateException e) {
            e2 = e;
        }
        if (gate1 != null && gate2 == null) {
            RaceBuild.addGate(currentCompletedGates, gate1, player);
        } else if (gate2 != null && gate1 == null) {
            RaceBuild.addGate(currentCompletedGates, gate2, player);
        } else if (gate1 != null) {
            gateChoices.put(userId, new RaceGate[]{gate1, gate2});
            RaceServer.error(player, "Two possible gates were found. Choose another block to specify which one you want to add.");
        } else {
            Object reason = "";
            if (!e1.endUserReason.isEmpty() && !e2.endUserReason.isEmpty()) {
                reason = "Either: " + e1.endUserReason + " OR " + e2.endUserReason;
            } else if (!e1.endUserReason.isEmpty()) {
                reason = e1.endUserReason;
            } else if (!e2.endUserReason.isEmpty()) {
                reason = e2.endUserReason;
            }
            RaceServer.error(player, "Failed to add a gate. " + (String)reason);
        }
    }

    private static void addGate(ArrayList<RaceGate> gates, RaceGate gate, ServerPlayer player) {
        gates.add(gate);
        RaceServer.success(player, "Gate " + gates.size() + " successfully added.");
        RaceNet.sendBuildGate(player, gate);
    }

    public static void completeTrack(String userId, ServerPlayer player) {
        if (!RaceBuild.checkBuildingMode(userId)) {
            return;
        }
        BlockKey startKey = startingBlock.get(userId);
        ArrayList<RaceGate> gates = completedGates.get(userId);
        String name = trackName.get(userId);
        if (gates.size() < 2) {
            RaceServer.error(player, "The track must contain at least 2 gates.");
            return;
        }
        if (name.isEmpty()) {
            RaceServer.error(player, "The track is unnamed. Set it using /race setName <name>.");
            return;
        }
        RaceBuild.exitBuildingMode(userId, player);
        RaceServer.then(RaceRepository.insertTrackWithGates(userId, name, startKey.dimension, startKey.pos, gates), raceTrackId -> RaceServer.success(player, "Track '" + name + "' successfully built!"), error -> RaceServer.error(player, "Unable to save the track. " + error.getMessage()));
    }

    public static void removeTrack(String userId, String dimension, BlockPos pos, ServerPlayer player) {
        BlockKey start = startingBlock.get(userId);
        if (start != null && pos.equals((Object)start.pos) && dimension.equals(start.dimension)) {
            RaceBuild.exitBuildingMode(userId, player);
        }
        if (!RaceServer.databaseReady) {
            return;
        }
        RaceServer.then(RaceRepository.softDeleteTrackAt(dimension, pos), track -> {
            if (track == null) {
                return;
            }
            RaceNavigate.exitRacingMode(userId, player, null);
            String reason = "The track starting point was removed by " + player.getPlainTextName();
            RaceNavigate.exitAllPlayersFromTrack(new BlockKey(track.dimension, track.getStartPos()), reason);
            if (userId.equals(track.ownerUserId)) {
                RaceServer.success(player, "Your track '" + track.name + "' was removed.");
            } else {
                RaceServer.info(player, "The track '" + track.name + "' was removed.");
            }
        }, error -> RaceServer.error(player, "Unable to remove the track. " + error.getMessage()));
    }

    private static ArrayList<BlockPos> findSolidNeighbors(BlockPos pos, Direction face, String dimension, BiPredicate<String, BlockPos> checkSolid) {
        Direction right;
        Direction up;
        if (face == Direction.UP || face == Direction.DOWN) {
            up = Direction.SOUTH;
            right = Direction.EAST;
        } else if (face == Direction.NORTH || face == Direction.SOUTH) {
            up = Direction.UP;
            right = Direction.EAST;
        } else {
            up = Direction.UP;
            right = Direction.SOUTH;
        }
        Direction down = up.getOpposite();
        Direction left = right.getOpposite();
        BlockPos topUp = RaceBuild.firstSolid(dimension, checkSolid, pos.relative(up).relative(face), pos.relative(up));
        BlockPos topRight = RaceBuild.firstSolid(dimension, checkSolid, pos.relative(right).relative(face), pos.relative(right));
        BlockPos topDown = RaceBuild.firstSolid(dimension, checkSolid, pos.relative(down).relative(face), pos.relative(down));
        BlockPos topLeft = RaceBuild.firstSolid(dimension, checkSolid, pos.relative(left).relative(face), pos.relative(left));
        ArrayList<BlockPos> result = new ArrayList<BlockPos>();
        if (topUp != null || topDown != null) {
            result.add(topUp != null ? topUp : topDown);
        }
        if (topRight != null || topLeft != null) {
            result.add(topRight != null ? topRight : topLeft);
        }
        return result;
    }

    private static BlockPos firstSolid(String dimension, BiPredicate<String, BlockPos> checkSolid, BlockPos ... tiers) {
        for (BlockPos p : tiers) {
            if (!checkSolid.test(dimension, p)) continue;
            return p;
        }
        return null;
    }

    public static void clear() {
        startingBlock.clear();
        gateChoices.clear();
        completedGates.clear();
        trackName.clear();
    }
}
