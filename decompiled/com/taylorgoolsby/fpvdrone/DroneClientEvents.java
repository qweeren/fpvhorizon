/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.blaze3d.platform.InputConstants
 *  com.mojang.blaze3d.platform.InputConstants$Type
 *  net.minecraft.client.KeyMapping
 *  net.minecraft.client.KeyMapping$Category
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.player.LocalPlayer
 *  net.minecraft.resources.Identifier
 *  net.minecraft.world.phys.Vec3
 *  net.minecraftforge.client.event.RegisterKeyMappingsEvent
 *  net.minecraftforge.client.event.ViewportEvent$ComputeCameraAngles
 *  net.minecraftforge.event.TickEvent$ClientTickEvent$Post
 *  net.minecraftforge.event.TickEvent$ClientTickEvent$Pre
 *  org.joml.Quaternionf
 */
package com.taylorgoolsby.fpvdrone;

import com.mojang.blaze3d.platform.InputConstants;
import com.taylorgoolsby.fpvdrone.DroneController;
import com.taylorgoolsby.fpvdrone.DroneHud;
import com.taylorgoolsby.fpvdrone.DroneNetwork;
import com.taylorgoolsby.fpvdrone.DroneState;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.resources.Identifier;
import net.minecraft.world.phys.Vec3;
import net.minecraftforge.client.event.RegisterKeyMappingsEvent;
import net.minecraftforge.client.event.ViewportEvent;
import net.minecraftforge.event.TickEvent;
import org.joml.Quaternionf;

public class DroneClientEvents {
    public static final KeyMapping.Category CATEGORY = KeyMapping.Category.register((Identifier)Identifier.fromNamespaceAndPath((String)"fpvdrone", (String)"main"));
    public static final KeyMapping TOGGLE_DRONE = new KeyMapping("key.fpvdrone.toggle", InputConstants.Type.KEYBOARD, 12, CATEGORY);
    private static final float YAW_RATE_DEG_PER_SEC = 180.0f;
    private static final float ROLL_PER_MOUSE_DEG = 1.0f;
    private static final float PITCH_PER_MOUSE_DEG = 1.0f;
    private static final float HUD_MOUSE_FULL_DEG = 6.0f;
    private static final float HUD_MOUSE_SMOOTHING = 0.35f;
    private static float hudMouseRoll;
    private static float hudMousePitch;

    public static void register() {
        RegisterKeyMappingsEvent.BUS.addListener(DroneClientEvents::onRegisterKeys);
        TickEvent.ClientTickEvent.Pre.BUS.addListener(DroneClientEvents::onClientTickPre);
        TickEvent.ClientTickEvent.Post.BUS.addListener(DroneClientEvents::onClientTick);
        ViewportEvent.ComputeCameraAngles.BUS.addListener(DroneClientEvents::onCameraAngles);
        DroneHud.register();
    }

    private static void onRegisterKeys(RegisterKeyMappingsEvent event) {
        event.register(TOGGLE_DRONE);
    }

    private static void onClientTickPre(TickEvent.ClientTickEvent.Pre event) {
        if (!DroneState.active) {
            return;
        }
        Minecraft mc = Minecraft.getInstance();
        mc.options.keyUp.setDown(false);
        mc.options.keyDown.setDown(false);
        mc.options.keyLeft.setDown(false);
        mc.options.keyRight.setDown(false);
        mc.options.keyJump.setDown(false);
        mc.options.keyShift.setDown(false);
    }

    private static void onClientTick(TickEvent.ClientTickEvent.Post event) {
        Minecraft mc = Minecraft.getInstance();
        LocalPlayer player = mc.player;
        DroneController.tick(mc);
        if (player == null) {
            return;
        }
        if (DroneState.active && player.isDeadOrDying()) {
            DroneState.exit(player);
            while (TOGGLE_DRONE.consumeClick()) {
            }
            return;
        }
        while (TOGGLE_DRONE.consumeClick()) {
            if (DroneState.active) {
                DroneState.exit(player);
            } else if (!player.isDeadOrDying()) {
                DroneState.enter(player);
            }
            player.refreshDimensions();
        }
        if (DroneState.active) {
            Quaternionf camera = DroneState.getCameraOrientation(new Quaternionf());
            DroneNetwork.sendOrientation(camera.x, camera.y, camera.z, camera.w);
        }
    }

    private static boolean physicallyDown(KeyMapping key) {
        return !key.isUnbound() && InputConstants.isKeyDown((int)key.getKey().getValue());
    }

    private static void onCameraAngles(ViewportEvent.ComputeCameraAngles event) {
        Vec3 position;
        if (!DroneState.active) {
            return;
        }
        Minecraft mc = Minecraft.getInstance();
        LocalPlayer player = mc.player;
        if (player == null) {
            return;
        }
        long now = System.nanoTime();
        float dt = Math.min((float)(now - DroneState.lastNanos) / 1.0E9f, 0.1f);
        DroneState.lastNanos = now;
        if (mc.gui.screen() != null) {
            return;
        }
        float mouseX = player.getYRot() - DroneState.lockedYaw;
        float mouseY = player.getXRot() - 0.0f;
        DroneState.pinPlayerHead(player);
        float yawInput = (DroneClientEvents.physicallyDown(mc.options.keyRight) ? 1.0f : 0.0f) - (DroneClientEvents.physicallyDown(mc.options.keyLeft) ? 1.0f : 0.0f);
        float throttleInput = (DroneClientEvents.physicallyDown(mc.options.keyUp) ? 1.0f : 0.0f) - (DroneClientEvents.physicallyDown(mc.options.keyDown) ? 1.0f : 0.0f);
        hudMouseRoll += (DroneClientEvents.clamp1(mouseX / 6.0f) - hudMouseRoll) * 0.35f;
        hudMousePitch += (DroneClientEvents.clamp1(mouseY / 6.0f) - hudMousePitch) * 0.35f;
        float hudYaw = yawInput;
        float hudRoll = hudMouseRoll;
        float hudPitch = hudMousePitch;
        float hudThrottle = DroneState.throttleToHud(DroneState.throttle);
        float pitchDeg = mouseY * 1.0f;
        float rollDeg = mouseX * 1.0f;
        float yawDeg = yawInput * 180.0f * dt;
        if (DroneController.poll()) {
            if (DroneController.isVirtual()) {
                yawDeg = 0.0f;
                throttleInput = 0.0f;
                hudYaw = 0.0f;
            }
            hudYaw += DroneController.rawYaw;
            hudRoll += DroneController.rawRoll;
            hudPitch += DroneController.rawPitch;
            hudThrottle = DroneController.rawThrottle;
            pitchDeg += DroneController.pitchDegPerSec * dt;
            rollDeg += DroneController.rollDegPerSec * dt;
            yawDeg += DroneController.yawDegPerSec * dt;
            DroneState.throttle = DroneController.throttle;
            throttleInput = 0.0f;
        }
        DroneState.orientation.rotateX(-((float)Math.toRadians(pitchDeg))).rotateZ(-((float)Math.toRadians(rollDeg))).rotateY(-((float)Math.toRadians(yawDeg))).normalize();
        DroneState.position = position = DroneState.stepPhysics(player, dt, throttleInput);
        DroneState.hudYaw = DroneClientEvents.clamp1(hudYaw);
        DroneState.hudRoll = DroneClientEvents.clamp1(hudRoll);
        DroneState.hudPitch = DroneClientEvents.clamp1(hudPitch);
        DroneState.hudThrottle = DroneClientEvents.clamp1(hudThrottle);
    }

    private static float clamp1(float v) {
        return Math.max(-1.0f, Math.min(1.0f, v));
    }
}
