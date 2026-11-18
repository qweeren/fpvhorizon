/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.player.LocalPlayer
 *  net.minecraft.world.phys.Vec3
 *  org.joml.Quaternionf
 *  org.joml.Vector3f
 */
package com.taylorgoolsby.fpvdrone.race.client;

import com.taylorgoolsby.fpvdrone.DroneState;
import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.phys.Vec3;
import org.joml.Quaternionf;
import org.joml.Vector3f;

final class RaceProjection {
    private RaceProjection() {
    }

    static Result project(double wx, double wy, double wz, float partialTick, int guiWidth, int guiHeight, float margin) {
        boolean inside;
        float vy;
        float vx;
        boolean inFront;
        Minecraft mc = Minecraft.getInstance();
        LocalPlayer player = mc.player;
        if (player == null) {
            return null;
        }
        Vec3 eye = player.getEyePosition(partialTick);
        Quaternionf rotation = new Quaternionf();
        if (DroneState.active) {
            DroneState.getCameraOrientation(rotation);
        } else {
            float yaw = player.getViewYRot(partialTick) * ((float)Math.PI / 180);
            float pitch = player.getViewXRot(partialTick) * ((float)Math.PI / 180);
            rotation.rotationYXZ((float)Math.PI - yaw, -pitch, 0.0f);
        }
        Vector3f d = new Vector3f((float)(wx - eye.x), (float)(wy - eye.y), (float)(wz - eye.z));
        double distance = d.length();
        rotation.transformInverse(d);
        double fovRad = Math.toRadians(((Integer)mc.options.fov().get()).intValue());
        float tanHalf = (float)Math.tan(fovRad / 2.0);
        float aspect = (float)guiWidth / (float)guiHeight;
        float cx = (float)guiWidth / 2.0f;
        float cy = (float)guiHeight / 2.0f;
        float depth = -d.z;
        boolean bl = inFront = depth > 0.05f;
        if (inFront) {
            vx = d.x / (depth * tanHalf * aspect) * cx;
            vy = -(d.y / (depth * tanHalf)) * cy;
        } else {
            vx = d.x;
            vy = -d.y;
            float len = (float)Math.sqrt(vx * vx + vy * vy);
            if (len < 1.0E-4f) {
                vx = 0.0f;
                vy = cy;
            } else {
                vx = vx / len * 10000.0f;
                vy = vy / len * 10000.0f;
            }
        }
        float halfW = cx - margin;
        float halfH = cy - margin;
        boolean bl2 = inside = inFront && Math.abs(vx) <= halfW && Math.abs(vy) <= halfH;
        if (!inside) {
            float sx = Math.abs(vx) > 1.0E-4f ? halfW / Math.abs(vx) : Float.MAX_VALUE;
            float sy = Math.abs(vy) > 1.0E-4f ? halfH / Math.abs(vy) : Float.MAX_VALUE;
            float s = Math.min(sx, sy);
            vx *= s;
            vy *= s;
        }
        return new Result(cx + vx, cy + vy, inside, distance);
    }

    record Result(float x, float y, boolean onScreen, double distance) {
    }
}
