/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.player.LocalPlayer
 *  net.minecraft.world.entity.MoverType
 *  net.minecraft.world.phys.Vec3
 *  org.joml.Quaternionf
 *  org.joml.Quaternionfc
 *  org.joml.Vector3f
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.AdvancedPhysicsCore;
import com.taylorgoolsby.fpvdrone.DroneBuild;
import com.taylorgoolsby.fpvdrone.DroneCollision;
import com.taylorgoolsby.fpvdrone.DroneController;
import com.taylorgoolsby.fpvdrone.DroneNetwork;
import com.taylorgoolsby.fpvdrone.DroneOrientations;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import com.taylorgoolsby.fpvdrone.race.V3;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.entity.MoverType;
import net.minecraft.world.phys.Vec3;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;
import org.joml.Vector3f;

public class DroneState {
    public static boolean active = false;
    public static final float THROTTLE_RATE = 0.6f;
    private static final AdvancedPhysicsCore physics = new AdvancedPhysicsCore();
    public static final Quaternionf orientation = new Quaternionf();
    public static Vec3 position = Vec3.ZERO;
    public static final Vector3f velocity = new Vector3f();
    public static float throttle;
    public static float hudRoll;
    public static float hudPitch;
    public static float hudYaw;
    public static float hudThrottle;
    public static final float LOCKED_PITCH = 0.0f;
    public static float lockedYaw;
    private static float origYaw;
    private static float origPitch;
    public static long lastNanos;

    private static float cameraTiltRad() {
        return (float)Math.toRadians(DroneBuild.get(DroneBuild.Param.CAMERA_ANGLE));
    }

    public static float throttleToHud(float throttle) {
        return DroneController.isMode3d() ? throttle : throttle * 2.0f - 1.0f;
    }

    public static Quaternionf getCameraOrientation(Quaternionf dest) {
        return dest.set((Quaternionfc)orientation).rotateX(DroneState.cameraTiltRad());
    }

    public static void enter(LocalPlayer player) {
        active = true;
        DroneRegistry.set(player.getUUID(), true);
        origYaw = player.getYRot();
        origPitch = player.getXRot();
        lockedYaw = origYaw;
        orientation.rotationYXZ((float)Math.PI - (float)Math.toRadians(origYaw), -((float)Math.toRadians(origPitch)), 0.0f).rotateX(-DroneState.cameraTiltRad());
        position = player.position();
        velocity.zero();
        throttle = physics.getHoverThrottle();
        physics.reset(throttle);
        hudYaw = 0.0f;
        hudPitch = 0.0f;
        hudRoll = 0.0f;
        hudThrottle = DroneState.throttleToHud(throttle);
        player.refreshDimensions();
        DroneState.pinPlayerHead(player);
        DroneNetwork.sendToggle(true);
        lastNanos = System.nanoTime();
    }

    public static void exit(LocalPlayer player) {
        active = false;
        DroneRegistry.set(player.getUUID(), false);
        velocity.zero();
        player.refreshDimensions();
        DroneNetwork.sendToggle(false);
        player.setYRot(origYaw);
        player.setXRot(origPitch);
        player.yRotO = origYaw;
        player.xRotO = origPitch;
    }

    public static void reset() {
        active = false;
        velocity.zero();
        DroneRegistry.clear();
        DroneOrientations.clear();
    }

    public static void pinPlayerHead(LocalPlayer player) {
        player.setYRot(lockedYaw);
        player.setXRot(0.0f);
        player.yRotO = lockedYaw;
        player.xRotO = 0.0f;
    }

    public static Vec3 stepPhysics(LocalPlayer player, float dt, float throttleInput) {
        boolean collided;
        float minThrottle = DroneController.isMode3d() ? -1.0f : 0.0f;
        throttle = Math.max(minThrottle, Math.min(1.0f, throttle + throttleInput * 0.6f * dt));
        physics.step(dt, throttle, orientation, velocity);
        Vec3 requestedMovement = new Vec3((double)(DroneState.velocity.x * dt), (double)(DroneState.velocity.y * dt), (double)(DroneState.velocity.z * dt));
        V3 preCollisionVelocity = new V3(DroneState.velocity.x, DroneState.velocity.y, DroneState.velocity.z);
        Vec3 before = player.position();
        player.move(MoverType.SELF, requestedMovement);
        Vec3 clippedMovement = player.position().subtract(before);
        boolean bl = collided = Math.abs(clippedMovement.x - requestedMovement.x) > 1.0E-5 || Math.abs(clippedMovement.y - requestedMovement.y) > 1.0E-5 || Math.abs(clippedMovement.z - requestedMovement.z) > 1.0E-5;
        if (collided) {
            DroneCollision.Result bounce = DroneCollision.resolve(new V3((float)requestedMovement.x, (float)requestedMovement.y, (float)requestedMovement.z), new V3((float)clippedMovement.x, (float)clippedMovement.y, (float)clippedMovement.z), preCollisionVelocity);
            velocity.set(bounce.velocity().x, bounce.velocity().y, bounce.velocity().z);
            Vec3 reflected = new Vec3((double)bounce.displacement().x - clippedMovement.x, (double)bounce.displacement().y - clippedMovement.y, (double)bounce.displacement().z - clippedMovement.z);
            if (reflected.lengthSqr() > 1.0E-10) {
                player.move(MoverType.SELF, reflected);
            }
        }
        position = player.position();
        return position;
    }
}
