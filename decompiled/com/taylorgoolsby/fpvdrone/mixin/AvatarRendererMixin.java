/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.blaze3d.vertex.PoseStack
 *  net.minecraft.client.renderer.entity.player.AvatarRenderer
 *  net.minecraft.client.renderer.entity.state.AvatarRenderState
 *  org.spongepowered.asm.mixin.Mixin
 *  org.spongepowered.asm.mixin.injection.At
 *  org.spongepowered.asm.mixin.injection.Inject
 *  org.spongepowered.asm.mixin.injection.callback.CallbackInfo
 */
package com.taylorgoolsby.fpvdrone.mixin;

import com.mojang.blaze3d.vertex.PoseStack;
import com.taylorgoolsby.fpvdrone.DroneDebug;
import com.taylorgoolsby.fpvdrone.DroneHeadRender;
import net.minecraft.client.renderer.entity.player.AvatarRenderer;
import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value={AvatarRenderer.class})
public abstract class AvatarRendererMixin {
    @Inject(method={"scale(Lnet/minecraft/client/renderer/entity/state/AvatarRenderState;Lcom/mojang/blaze3d/vertex/PoseStack;)V"}, at={@At(value="TAIL")})
    private void fpvdrone$centerHead(AvatarRenderState state, PoseStack poseStack, CallbackInfo ci) {
        boolean drone = DroneHeadRender.isDroneAvatar(state);
        DroneDebug.markHook(drone);
        if (drone) {
            DroneHeadRender.positionHead(state, poseStack);
        }
    }
}
