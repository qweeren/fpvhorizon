/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.race.V3;

public final class DroneCollision {
    public static final float ELASTIC = 0.2f;
    public static final float GLANCING_KEEP = 0.65f;
    public static final float MIN_BOUNCE_SPEED = 1.0f;

    private DroneCollision() {
    }

    public static Result resolve(V3 displacement, V3 clipped, V3 preCollisionVelocity) {
        float desiredSpeed = preCollisionVelocity.length();
        V3 vClip = clipped.sub(displacement);
        float angle = displacement.normalized().angleBetween(vClip.mul(-1.0f).normalized());
        V3 cProj = clipped.project(vClip.normalized());
        if (Float.isNaN(cProj.x)) {
            return new Result(V3.ZERO, clipped);
        }
        V3 hClip = cProj.sub(clipped).normalized().mul(vClip.length() * (float)Math.tan(angle));
        V3 puncture = clipped.add(hClip);
        V3 bounceFinal = clipped.add(vClip);
        V3 bounceDirection = bounceFinal.sub(puncture).normalized();
        float glancing = (float)Math.sin(angle);
        float outSpeed = desiredSpeed * (glancing * 0.65f + (1.0f - glancing) * 0.2f);
        if (outSpeed < 1.0f) {
            return new Result(V3.ZERO, clipped);
        }
        return new Result(bounceDirection.mul(outSpeed), bounceFinal);
    }

    public record Result(V3 velocity, V3 displacement) {
    }
}
