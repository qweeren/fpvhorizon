/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.net.GateBox;
import com.taylorgoolsby.fpvdrone.race.net.LeaderboardEntry;
import java.util.ArrayList;
import java.util.List;

public final class RaceClientState {
    public static volatile boolean buildMode = false;
    public static volatile String buildDimension = "";
    public static volatile List<GateBox> buildGates = List.of();
    public static volatile boolean racing = false;
    public static volatile String trackName = "";
    public static volatile String dimension = "";
    public static volatile List<GateBox> gates = List.of();
    public static volatile int nextGate = 0;
    public static volatile long lapStartMillis = 0L;
    public static volatile int lastLapMillis = -1;
    public static volatile List<LeaderboardEntry> leaderboard = List.of();

    private RaceClientState() {
    }

    public static void setBuildMode(boolean active) {
        buildMode = active;
        buildGates = List.of();
        buildDimension = "";
    }

    public static void addBuildGate(String dim, GateBox gate) {
        if (!dim.equals(buildDimension)) {
            buildGates = List.of();
            buildDimension = dim;
        }
        ArrayList<GateBox> next = new ArrayList<GateBox>(buildGates);
        next.add(gate);
        buildGates = List.copyOf(next);
    }

    public static void startRacing(String name, String dim, List<GateBox> trackGates) {
        racing = true;
        trackName = name;
        dimension = dim;
        gates = List.copyOf(trackGates);
        nextGate = 0;
        lapStartMillis = 0L;
        lastLapMillis = -1;
        leaderboard = List.of();
    }

    public static void stopRacing() {
        racing = false;
        trackName = "";
        dimension = "";
        gates = List.of();
        nextGate = 0;
        lapStartMillis = 0L;
        lastLapMillis = -1;
        leaderboard = List.of();
    }

    public static void setNextGate(int index) {
        nextGate = index;
    }

    public static void startLap(int completedLapMillis) {
        lapStartMillis = System.currentTimeMillis();
        if (completedLapMillis >= 0) {
            lastLapMillis = completedLapMillis;
        }
    }

    public static void setLeaderboard(List<LeaderboardEntry> entries) {
        leaderboard = List.copyOf(entries);
    }

    public static GateBox getNextGateBox() {
        List<GateBox> g = gates;
        int i = nextGate;
        if (!racing || i < 0 || i >= g.size()) {
            return null;
        }
        return g.get(i);
    }

    public static void reset() {
        RaceClientState.setBuildMode(false);
        RaceClientState.stopRacing();
    }
}
