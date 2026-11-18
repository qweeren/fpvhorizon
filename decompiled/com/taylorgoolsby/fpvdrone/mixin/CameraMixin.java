/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Camera
 *  org.joml.Quaternionf
 *  org.joml.Quaternionfc
 *  org.joml.Vector3f
 *  org.spongepowered.asm.mixin.Final
 *  org.spongepowered.asm.mixin.Mixin
 *  org.spongepowered.asm.mixin.Shadow
 *  org.spongepowered.asm.mixin.injection.At
 *  org.spongepowered.asm.mixin.injection.Inject
 *  org.spongepowered.asm.mixin.injection.callback.CallbackInfo
 */
package com.taylorgoolsby.fpvdrone.mixin;

import com.taylorgoolsby.fpvdrone.DroneState;
import net.minecraft.client.Camera;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value={Camera.class})
public abstract class CameraMixin {
    @Shadow
    @Final
    private Quaternionf rotation;
    @Shadow
    @Final
    private Vector3f forwards;
    @Shadow
    @Final
    private Vector3f up;
    @Shadow
    @Final
    private Vector3f left;
    @Shadow
    private int matrixPropertiesDirty;

    @Inject(method={"setRotation(FFF)V"}, at={@At(value="TAIL")})
    private void fpvdrone$applyDroneOrientation(float yRot, float xRot, float z, CallbackInfo ci) {
        if (!DroneState.active) {
            return;
        }
        DroneState.getCameraOrientation(this.rotation);
        new Vector3f(0.0f, 0.0f, -1.0f).rotate((Quaternionfc)this.rotation, this.forwards);
        new Vector3f(0.0f, 1.0f, 0.0f).rotate((Quaternionfc)this.rotation, this.up);
        new Vector3f(-1.0f, 0.0f, 0.0f).rotate((Quaternionfc)this.rotation, this.left);
        this.matrixPropertiesDirty |= 3;
    }
}
