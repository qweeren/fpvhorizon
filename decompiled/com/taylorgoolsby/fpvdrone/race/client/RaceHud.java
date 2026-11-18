/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.DeltaTracker
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.gui.Font
 *  net.minecraft.client.gui.GuiGraphicsExtractor
 *  net.minecraft.network.chat.Component
 *  net.minecraft.resources.Identifier
 *  net.minecraftforge.client.event.AddGuiOverlayLayersEvent
 */
package com.taylorgoolsby.fpvdrone.race.client;

import com.taylorgoolsby.fpvdrone.race.RaceClientState;
import com.taylorgoolsby.fpvdrone.race.RaceTime;
import com.taylorgoolsby.fpvdrone.race.client.RaceProjection;
import com.taylorgoolsby.fpvdrone.race.net.GateBox;
import com.taylorgoolsby.fpvdrone.race.net.LeaderboardEntry;
import java.util.List;
import java.util.Objects;
import java.util.UUID;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;
import net.minecraftforge.client.event.AddGuiOverlayLayersEvent;

final class RaceHud {
    private static final Identifier LAYER_ID = Identifier.fromNamespaceAndPath((String)"fpvdrone", (String)"race_hud");
    private static final int PANEL_WIDTH = 150;
    private static final int PANEL_MARGIN = 6;
    private static final int PAD = 4;
    private static final int LINE_HEIGHT = 10;
    private static final int COLOR_PANEL = -2013265920;
    private static final int COLOR_TEXT = -1;
    private static final int COLOR_DIM = -5592406;
    private static final int COLOR_TIMER = -11930;
    private static final int COLOR_SELF = -11141291;
    private static final int COLOR_LEADER = -11930;
    private static final int COLOR_MARKER = -11930;
    private static final int COLOR_MARKER_OUTLINE = -15261656;
    private static final int MARKER_RADIUS = 6;

    private RaceHud() {
    }

    static void register() {
        AddGuiOverlayLayersEvent.BUS.addListener(RaceHud::onAddLayers);
    }

    private static void onAddLayers(AddGuiOverlayLayersEvent event) {
        event.getLayeredDraw().addWithCondition(LAYER_ID, RaceHud::render, () -> RaceClientState.racing || RaceClientState.buildMode);
    }

    private static void render(GuiGraphicsExtractor gg, DeltaTracker deltaTracker) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null || mc.level == null) {
            return;
        }
        if (RaceClientState.racing) {
            RaceHud.drawRacingPanel(gg, mc);
            RaceHud.drawGateMarker(gg, mc, deltaTracker.getGameTimeDeltaPartialTick(false));
        } else if (RaceClientState.buildMode) {
            RaceHud.drawBuildPanel(gg, mc);
        }
    }

    private static void drawBuildPanel(GuiGraphicsExtractor gg, Minecraft mc) {
        int width = mc.getWindow().getGuiScaledWidth();
        int left = width - 150 - 6;
        int top = 6;
        gg.fill(left, top, left + 150, top + 8 + 20, -2013265920);
        RaceHud.text(gg, mc.font, "Building a track", left + 4, top + 4, -11930);
        RaceHud.text(gg, mc.font, RaceClientState.buildGates.size() + " gates added", left + 4, top + 4 + 10, -1);
    }

    private static void drawRacingPanel(GuiGraphicsExtractor gg, Minecraft mc) {
        Font font = mc.font;
        int width = mc.getWindow().getGuiScaledWidth();
        int left = width - 150 - 6;
        int top = 6;
        List<LeaderboardEntry> board = RaceClientState.leaderboard;
        int lines = 4 + (board.isEmpty() ? 0 : 1 + board.size());
        gg.fill(left, top, left + 150, top + 8 + 10 * lines + 2, -2013265920);
        int x = left + 4;
        int y = top + 4;
        RaceHud.text(gg, font, RaceClientState.trackName, x, y, -1);
        long start = RaceClientState.lapStartMillis;
        String lap = start == 0L ? "--:--.--" : RaceTime.format(System.currentTimeMillis() - start);
        RaceHud.text(gg, font, "Lap  " + lap, x, y += 10, -11930);
        int last = RaceClientState.lastLapMillis;
        RaceHud.text(gg, font, "Last " + (last < 0 ? "--:--.--" : RaceTime.format(last)), x, y += 10, -5592406);
        int total = RaceClientState.gates.size();
        String gateText = start == 0L ? "Fly through gate 1 to start" : "Next gate " + (RaceClientState.nextGate + 1) + " / " + total;
        RaceHud.text(gg, font, gateText, x, y += 10, -5592406);
        y += 10;
        if (!board.isEmpty()) {
            RaceHud.text(gg, font, "Best laps", x, y += 2, -5592406);
            y += 10;
            UUID self = mc.player.getUUID();
            for (int i = 0; i < board.size(); ++i) {
                LeaderboardEntry e = board.get(i);
                int color = e.userId().equals(self) ? -11141291 : (i == 0 ? -11930 : -1);
                String time = RaceTime.format(e.millis());
                int maxNameWidth = 142 - font.width(time) - 24;
                RaceHud.text(gg, font, i + 1 + ". " + RaceHud.fit(font, e.name(), maxNameWidth), x, y, color);
                RaceHud.text(gg, font, time, left + 150 - 4 - font.width(time), y, color);
                y += 10;
            }
        }
    }

    private static void drawGateMarker(GuiGraphicsExtractor gg, Minecraft mc, float partialTick) {
        GateBox gate = RaceClientState.getNextGateBox();
        if (gate == null) {
            return;
        }
        if (!RaceClientState.dimension.equals(mc.level.dimension().identifier().toString())) {
            return;
        }
        int w = mc.getWindow().getGuiScaledWidth();
        int h = mc.getWindow().getGuiScaledHeight();
        RaceProjection.Result r = RaceProjection.project(gate.centerX(), gate.centerY(), gate.centerZ(), partialTick, w, h, 20.0f);
        if (r == null) {
            return;
        }
        int x = Math.round(r.x());
        int y = Math.round(r.y());
        RaceHud.drawDiamond(gg, x, y, 7, -15261656);
        RaceHud.drawDiamond(gg, x, y, 6, -11930);
        String label = Math.round(r.distance()) + " m";
        Font font = mc.font;
        int labelX = x - font.width(label) / 2;
        int labelY = y + 6 + 4;
        Objects.requireNonNull(font);
        if (labelY + 9 > h) {
            Objects.requireNonNull(font);
            labelY = y - 6 - 4 - 9;
        }
        RaceHud.text(gg, font, label, labelX, labelY, -1);
    }

    private static void drawDiamond(GuiGraphicsExtractor gg, int cx, int cy, int radius, int color) {
        for (int dy = -radius; dy <= radius; ++dy) {
            int half = radius - Math.abs(dy);
            gg.fill(cx - half, cy + dy, cx + half + 1, cy + dy + 1, color);
        }
    }

    private static String fit(Font font, String s, int maxWidth) {
        while (s.length() > 1 && font.width(s) > maxWidth) {
            s = s.substring(0, s.length() - 1);
        }
        return s;
    }

    private static void text(GuiGraphicsExtractor gg, Font font, String s, int x, int y, int color) {
        gg.text(font, Component.literal((String)s).getVisualOrderText(), x, y, color);
    }
}
