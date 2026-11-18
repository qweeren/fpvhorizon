/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.DeltaTracker
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.gui.GuiGraphicsExtractor
 *  net.minecraft.resources.Identifier
 *  net.minecraftforge.client.event.AddGuiOverlayLayersEvent
 *  net.minecraftforge.client.gui.overlay.ForgeLayeredDraw
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneState;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.resources.Identifier;
import net.minecraftforge.client.event.AddGuiOverlayLayersEvent;
import net.minecraftforge.client.gui.overlay.ForgeLayeredDraw;

public final class DroneHud {
    private static final Identifier LAYER_ID = Identifier.fromNamespaceAndPath((String)"fpvdrone", (String)"stick_overlay");
    private static final int BOX_SIZE = 56;
    private static final int MARGIN_X = 14;
    private static final int MARGIN_Y = 14;
    private static final int MARKER_SIZE = 5;
    private static final int COLOR_CROSSHAIR = -1711276033;
    private static final int COLOR_MARKER_BORDER = -1711276033;

    private DroneHud() {
    }

    public static void register() {
        AddGuiOverlayLayersEvent.BUS.addListener(DroneHud::onAddLayers);
    }

    private static void onAddLayers(AddGuiOverlayLayersEvent event) {
        DroneHud.hideVanillaLayersWhileFlying(event.getLayeredDraw());
        event.getLayeredDraw().addWithCondition(LAYER_ID, DroneHud::render, () -> DroneState.active);
    }

    private static void hideVanillaLayersWhileFlying(ForgeLayeredDraw draw) {
        DroneHud.hide(draw, ForgeLayeredDraw.PRE_SLEEP_STACK, ForgeLayeredDraw.CROSSHAIR, ForgeLayeredDraw.POTION_EFFECTS, ForgeLayeredDraw.BOSS_OVERLAY);
        DroneHud.hide(draw, ForgeLayeredDraw.HOTBAR_AND_DECOS, ForgeLayeredDraw.SPECTATOR_HOTBAR, ForgeLayeredDraw.ITEM_HOTBAR, ForgeLayeredDraw.HEALTH_BAR, ForgeLayeredDraw.VEHICLE_HEALTH, ForgeLayeredDraw.BACKGROUND, ForgeLayeredDraw.EXPERIENCE_LEVEL, ForgeLayeredDraw.CONTEXTUAL_INFO, ForgeLayeredDraw.SELECTED_ITEM_NAME, ForgeLayeredDraw.SPECTATOR_ACTION);
        DroneHud.hide(draw, ForgeLayeredDraw.POST_SLEEP_STACK, ForgeLayeredDraw.DEMO_OVERLAY, ForgeLayeredDraw.SCOREBOARD, ForgeLayeredDraw.HOTBAR_MESSAGE, ForgeLayeredDraw.TITLE_OVERLAY);
    }

    private static void hide(ForgeLayeredDraw draw, Identifier stack, Identifier ... layers) {
        for (Identifier layer : layers) {
            draw.addConditionTo(stack, layer, () -> !DroneState.active);
        }
    }

    private static void render(GuiGraphicsExtractor gg, DeltaTracker deltaTracker) {
        Minecraft mc = Minecraft.getInstance();
        int width = mc.getWindow().getGuiScaledWidth();
        int height = mc.getWindow().getGuiScaledHeight();
        int top = height - 14 - 56;
        DroneHud.drawGimbal(gg, 14, top, DroneState.hudYaw, DroneState.hudThrottle);
        DroneHud.drawGimbal(gg, width - 14 - 56, top, DroneState.hudRoll, DroneState.hudPitch);
    }

    private static void drawGimbal(GuiGraphicsExtractor gg, int left, int top, float x, float y) {
        int right = left + 56;
        int bottom = top + 56;
        int cx = left + 28;
        int cy = top + 28;
        gg.fill(left, cy, right, cy + 1, -1711276033);
        gg.fill(cx, top, cx + 1, bottom, -1711276033);
        int half = 3;
        float travel = 27.0f - (float)half;
        int mx = cx + Math.round(DroneHud.clamp(x) * travel);
        int my = cy - Math.round(DroneHud.clamp(y) * travel);
        int l = mx - half;
        int r = mx + half + 1;
        int t = my - half;
        int bt = my + half + 1;
        gg.fill(l, t, r, t + 1, -1711276033);
        gg.fill(l, bt - 1, r, bt, -1711276033);
        gg.fill(l, t + 1, l + 1, bt - 1, -1711276033);
        gg.fill(r - 1, t + 1, r, bt - 1, -1711276033);
    }

    private static float clamp(float v) {
        return Math.max(-1.0f, Math.min(1.0f, v));
    }
}
