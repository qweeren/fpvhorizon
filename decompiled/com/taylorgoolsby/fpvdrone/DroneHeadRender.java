/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.blaze3d.vertex.PoseStack
 *  com.mojang.math.Axis
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.multiplayer.ClientLevel
 *  net.minecraft.client.renderer.entity.state.AvatarRenderState
 *  net.minecraft.world.entity.Entity
 *  net.minecraft.world.entity.Pose
 *  net.minecraft.world.entity.player.Player
 *  net.minecraftforge.client.event.RenderAvatarEvent$Pre
 *  org.joml.Quaternionf
 *  org.joml.Quaternionfc
 *  org.joml.Vector3f
 */
package com.taylorgoolsby.fpvdrone;

import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Axis;
import com.taylorgoolsby.fpvdrone.DroneDebug;
import com.taylorgoolsby.fpvdrone.DroneOrientations;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import com.taylorgoolsby.fpvdrone.DroneState;
import java.util.UUID;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.player.Player;
import net.minecraftforge.client.event.RenderAvatarEvent;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;
import org.joml.Vector3f;

public final class DroneHeadRender {
    private static final float HEAD_CENTER_ABOVE_ORIGIN = 1.751f;
    private static final float MODEL_SCALE = 0.9375f;
    private static final float DRONE_BOX_HEIGHT = 0.3f;

    private DroneHeadRender() {
    }

    public static void register() {
        RenderAvatarEvent.Pre.BUS.addListener(DroneHeadRender::onPre);
    }

    private static UUID droneUuid(AvatarRenderState state) {
        Player player;
        ClientLevel level = Minecraft.getInstance().level;
        if (level == null) {
            return null;
        }
        Entity entity = level.getEntity(state.id);
        if (entity instanceof Player && DroneRegistry.isDrone((player = (Player)entity).getUUID())) {
            return player.getUUID();
        }
        return null;
    }

    private static void ensureDroneHitbox(AvatarRenderState state) {
        Player player;
        ClientLevel level = Minecraft.getInstance().level;
        if (level == null) {
            return;
        }
        Entity entity = level.getEntity(state.id);
        if (entity instanceof Player && Math.abs((player = (Player)entity).getBbHeight() - 0.3f) > 0.001f) {
            player.refreshDimensions();
        }
    }

    public static boolean isDroneAvatar(AvatarRenderState state) {
        return DroneHeadRender.droneUuid(state) != null;
    }

    private static boolean hasCameraOrientation(AvatarRenderState state, UUID uuid) {
        return DroneHeadRender.isLocalDrone(state) || DroneOrientations.has(uuid);
    }

    private static boolean cameraOrientation(AvatarRenderState state, UUID uuid, Quaternionf dest) {
        if (DroneHeadRender.isLocalDrone(state)) {
            DroneState.getCameraOrientation(dest);
            return true;
        }
        return DroneOrientations.get(uuid, dest);
    }

    private static void onPre(RenderAvatarEvent.Pre event) {
        AvatarRenderState state = event.getState();
        UUID uuid = DroneHeadRender.droneUuid(state);
        if (uuid == null) {
            return;
        }
        DroneHeadRender.ensureDroneHitbox(state);
        if (DroneDebug.isEnabled()) {
            ClientLevel level = Minecraft.getInstance().level;
            Entity entity = level == null ? null : level.getEntity(state.id);
            DroneDebug.markRender(uuid, entity == null ? "?" : entity.getName().getString(), DroneHeadRender.isLocalDrone(state), DroneOrientations.has(uuid), state.boundingBoxHeight);
        }
        state.isSpectator = true;
        state.pose = Pose.STANDING;
        state.isCrouching = false;
        state.isFallFlying = false;
        state.isVisuallySwimming = false;
        state.isAutoSpinAttack = false;
        state.swimAmount = 0.0f;
        if (DroneHeadRender.hasCameraOrientation(state, uuid)) {
            state.bodyRot = 0.0f;
            state.yRot = 0.0f;
            state.xRot = 0.0f;
        }
    }

    private static boolean isLocalDrone(AvatarRenderState state) {
        Minecraft mc = Minecraft.getInstance();
        return DroneState.active && mc.player != null && state.id == mc.player.getId();
    }

    public static void positionHead(AvatarRenderState state, PoseStack poseStack) {
        poseStack.translate(0.0f, DroneHeadRender.headShift(state), 0.0f);
        UUID uuid = DroneHeadRender.droneUuid(state);
        if (uuid == null) {
            return;
        }
        Quaternionf camera = new Quaternionf();
        if (!DroneHeadRender.cameraOrientation(state, uuid, camera)) {
            return;
        }
        Quaternionf turn = new Quaternionf().rotateX((float)Math.PI).mul((Quaternionfc)camera).rotateZ((float)Math.PI);
        Vector3f euler = turn.getEulerAnglesYXZ(new Vector3f());
        if (DroneDebug.isEnabled()) {
            DroneDebug.markApplied(uuid, (float)Math.toDegrees(euler.y), (float)Math.toDegrees(euler.x), (float)Math.toDegrees(euler.z));
        }
        poseStack.translate(0.0f, -1.751f, 0.0f);
        poseStack.rotate(Axis.YP, euler.y);
        poseStack.rotate(Axis.XP, euler.x);
        poseStack.rotate(Axis.ZP, euler.z);
        poseStack.translate(0.0f, 1.751f, 0.0f);
    }

    private static float headShift(AvatarRenderState state) {
        float hitboxCenter = 0.15f;
        return 1.751f - hitboxCenter / 0.9375f;
    }
}
