/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 *  net.minecraft.server.level.ServerPlayer
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.BlockKey;
import com.taylorgoolsby.fpvdrone.race.GateCrossing;
import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.RaceGateException;
import com.taylorgoolsby.fpvdrone.race.RaceNet;
import com.taylorgoolsby.fpvdrone.race.RaceServer;
import com.taylorgoolsby.fpvdrone.race.RaceTime;
import com.taylorgoolsby.fpvdrone.race.RaceTrack;
import com.taylorgoolsby.fpvdrone.race.RaceWaypoints;
import com.taylorgoolsby.fpvdrone.race.db.RaceRepository;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.function.BiPredicate;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerPlayer;

public final class RaceNavigate {
    private static final int LEADERBOARD_SIZE = 10;
    private static final long MAX_LAP_MILLIS = 0xFFFFFFL;
    private static final Map<String, RaceTrack> userToTrack = new HashMap<String, RaceTrack>();
    private static final Map<BlockKey, RaceTrack> startToTrack = new HashMap<BlockKey, RaceTrack>();
    private static final Map<BlockKey, ArrayList<String>> startToUsers = new HashMap<BlockKey, ArrayList<String>>();
    private static final Map<String, double[]> prevUserPos = new HashMap<String, double[]>();
    private static final Map<String, Integer> userNextGate = new HashMap<String, Integer>();
    private static final Map<String, Long> userStartTime = new HashMap<String, Long>();
    private static final Set<String> loading = new HashSet<String>();

    private RaceNavigate() {
    }

    public static boolean checkRacingMode(String userId) {
        return userToTrack.containsKey(userId);
    }

    public static void handleRightClickTrack(String userId, String dimension, BlockPos startingPos, BiPredicate<String, BlockPos> checkSolid, ServerPlayer player) {
        BlockKey startKey;
        RaceTrack loaded;
        if (!RaceServer.databaseReady) {
            RaceServer.databaseUnavailable(player);
            return;
        }
        if (loading.contains(userId)) {
            return;
        }
        if (RaceNavigate.checkRacingMode(userId)) {
            RaceTrack current = userToTrack.get(userId);
            if (current.dimension.equals(dimension) && current.getStartPos().equals((Object)startingPos)) {
                RaceNavigate.exitRacingMode(userId, player, null);
                return;
            }
            RaceNavigate.exitRacingMode(userId, player, null);
        }
        if ((loaded = startToTrack.get(startKey = new BlockKey(dimension, startingPos))) != null) {
            RaceNavigate.enterRacingMode(userId, loaded, startKey, player);
            return;
        }
        loading.add(userId);
        RaceServer.then(RaceRepository.getTrackWithGates(dimension, startingPos), track -> {
            loading.remove(userId);
            if (track == null) {
                return;
            }
            if (player.isRemoved() || RaceNavigate.checkRacingMode(userId)) {
                return;
            }
            RaceTrack existing = startToTrack.get(startKey);
            if (existing != null) {
                RaceNavigate.enterRacingMode(userId, existing, startKey, player);
                return;
            }
            try {
                track.computeGates(checkSolid);
            }
            catch (RaceGateException e) {
                RaceServer.error(player, e.endUserReason);
                return;
            }
            if (track.gates.size() < 2) {
                RaceServer.error(player, "This track has fewer than 2 gates.");
                return;
            }
            startToTrack.put(startKey, (RaceTrack)track);
            RaceNavigate.enterRacingMode(userId, track, startKey, player);
        }, error -> {
            loading.remove(userId);
            RaceServer.error(player, "Unable to load the track. " + error.getMessage());
        });
    }

    private static void enterRacingMode(String userId, RaceTrack track, BlockKey startKey, ServerPlayer player) {
        userToTrack.put(userId, track);
        userNextGate.put(userId, 0);
        userStartTime.remove(userId);
        prevUserPos.put(userId, new double[]{player.getX(), player.getY(), player.getZ()});
        startToUsers.computeIfAbsent(startKey, k -> new ArrayList()).add(userId);
        RaceServer.info(player, "You are racing track '" + track.name + "'.");
        RaceNet.sendRaceStart(player, track);
        RaceNet.sendNextGate(player, 0);
        RaceWaypoints.show(player, track.gates.get(0));
        RaceNavigate.sendLeaderboard(track, player);
    }

    private static void sendLeaderboard(RaceTrack track, ServerPlayer only) {
        RaceServer.then(RaceRepository.getBestTimes(track.raceTrackId, 10), laps -> {
            if (only != null) {
                if (RaceNavigate.checkRacingMode(RaceServer.getId(only.getUUID()))) {
                    RaceNet.sendLeaderboard(only, laps);
                }
                return;
            }
            ArrayList<String> users = startToUsers.get(new BlockKey(track.dimension, track.getStartPos()));
            if (users == null) {
                return;
            }
            for (String id : new ArrayList<String>(users)) {
                ServerPlayer p = RaceServer.getPlayer(id);
                if (p == null) continue;
                RaceNet.sendLeaderboard(p, laps);
            }
        }, error -> {});
    }

    public static void exitRacingMode(String userId, ServerPlayer player, String reason) {
        RaceTrack track = userToTrack.get(userId);
        if (track == null) {
            return;
        }
        BlockKey startKey = new BlockKey(track.dimension, track.getStartPos());
        userToTrack.remove(userId);
        userNextGate.remove(userId);
        userStartTime.remove(userId);
        prevUserPos.remove(userId);
        ArrayList<String> users = startToUsers.get(startKey);
        if (users != null) {
            users.remove(userId);
            if (users.isEmpty()) {
                startToUsers.remove(startKey);
                startToTrack.remove(startKey);
            }
        }
        if (player != null) {
            if (reason == null) {
                RaceServer.info(player, "You left racing mode.");
            } else {
                RaceServer.error(player, "You were kicked out of racing mode because: " + reason);
            }
            RaceNet.sendRaceStop(player);
            RaceWaypoints.hide(player);
        }
    }

    public static void exitAllPlayersFromTrack(BlockKey key, String reason) {
        ArrayList<String> userIds = startToUsers.get(key);
        if (userIds == null) {
            return;
        }
        for (String userId : new ArrayList<String>(userIds)) {
            RaceNavigate.exitRacingMode(userId, RaceServer.getPlayer(userId), reason);
        }
    }

    public static void changedGateBlock(String dimension, BlockPos pos, String reason) {
        if (startToTrack.isEmpty()) {
            return;
        }
        block0: for (Map.Entry<BlockKey, RaceTrack> entry : new ArrayList<Map.Entry<BlockKey, RaceTrack>>(startToTrack.entrySet())) {
            RaceTrack track = entry.getValue();
            if (!track.dimension.equals(dimension) || track.gates == null) continue;
            for (RaceGate gate : track.gates) {
                if (!gate.dimension.equals(dimension) || !gate.containsBlock(pos)) continue;
                RaceNavigate.exitAllPlayersFromTrack(entry.getKey(), reason);
                continue block0;
            }
        }
    }

    public static void onPlayerMoved(String userId, String dimension, double[] playerPos, ServerPlayer player) {
        boolean passedGate;
        double[] hit = new double[3];
        int loopCount = 0;
        do {
            if (!(passedGate = RaceNavigate.checkUserPassedGate(userId, dimension, playerPos, hit))) continue;
            RaceNavigate.handleGatePassed(userId, player);
            prevUserPos.put(userId, (double[])hit.clone());
        } while (passedGate && RaceNavigate.checkRacingMode(userId) && ++loopCount <= 150);
        if (RaceNavigate.checkRacingMode(userId)) {
            prevUserPos.put(userId, playerPos);
        }
    }

    private static boolean checkUserPassedGate(String userId, String dimension, double[] playerPos, double[] intersectionOut) {
        double[] prev = prevUserPos.get(userId);
        if (prev == null) {
            return false;
        }
        RaceGate nextGate = RaceNavigate.getNextGate(userId);
        if (nextGate == null) {
            return false;
        }
        return GateCrossing.passedThrough(nextGate, dimension, prev, playerPos, intersectionOut);
    }

    private static RaceGate getNextGate(String userId) {
        RaceTrack track = userToTrack.get(userId);
        if (track == null || track.gates == null) {
            return null;
        }
        Integer gateIndex = userNextGate.get(userId);
        if (gateIndex == null || gateIndex < 0 || gateIndex >= track.gates.size()) {
            return null;
        }
        return track.gates.get(gateIndex);
    }

    private static void handleGatePassed(String userId, ServerPlayer player) {
        RaceTrack track = userToTrack.get(userId);
        Integer gateIndex = userNextGate.get(userId);
        if (track == null || track.gates == null || gateIndex == null) {
            return;
        }
        int maxIndex = track.gates.size() - 1;
        int nextIndex = gateIndex + 1;
        if (nextIndex > maxIndex) {
            nextIndex = 0;
        }
        userNextGate.put(userId, nextIndex);
        RaceNet.sendNextGate(player, nextIndex);
        RaceWaypoints.show(player, track.gates.get(nextIndex));
        boolean quiet = RaceServer.isFlying(player.getUUID());
        if (gateIndex == 0) {
            if (userStartTime.get(userId) == null) {
                userStartTime.put(userId, System.currentTimeMillis());
                if (!quiet) {
                    RaceServer.info(player, "Passed gate " + (gateIndex + 1) + " of " + track.gates.size() + ".");
                }
                RaceNet.sendLapStart(player, -1);
            } else {
                RaceNavigate.handleLapComplete(userId, track, player, quiet);
            }
        } else if (!quiet) {
            RaceServer.info(player, "Passed gate " + (gateIndex + 1) + " of " + track.gates.size() + ".");
        }
    }

    private static void handleLapComplete(String userId, RaceTrack track, ServerPlayer player, boolean quiet) {
        long endMillis = System.currentTimeMillis();
        long elapsed = endMillis - userStartTime.get(userId);
        userStartTime.put(userId, endMillis);
        if (elapsed > 0xFFFFFFL) {
            RaceNet.sendLapStart(player, -1);
            RaceServer.error(player, "Lap discarded. Took too long.");
            return;
        }
        RaceNet.sendLapStart(player, (int)elapsed);
        if (!quiet) {
            RaceServer.success(player, "Lap time: " + RaceTime.format(elapsed));
        }
        RaceServer.then(RaceRepository.insertLap(track.raceTrackId, userId, player.getPlainTextName(), (int)elapsed, "26.3"), lapId -> RaceNavigate.sendLeaderboard(track, null), error -> RaceServer.error(player, "Unable to save your lap time. " + error.getMessage()));
    }

    public static void clear() {
        userToTrack.clear();
        startToTrack.clear();
        startToUsers.clear();
        prevUserPos.clear();
        userNextGate.clear();
        userStartTime.clear();
        loading.clear();
    }
}
