/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.renderer.FirstPersonHandsAndItemsRenderer
 *  org.spongepowered.asm.mixin.Mixin
 *  org.spongepowered.asm.mixin.injection.At
 *  org.spongepowered.asm.mixin.injection.Inject
 *  org.spongepowered.asm.mixin.injection.callback.CallbackInfo
 */
package com.taylorgoolsby.fpvdrone.mixin;

import com.taylorgoolsby.fpvdrone.DroneState;
import net.minecraft.client.renderer.FirstPersonHandsAndItemsRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value={FirstPersonHandsAndItemsRenderer.class})
public abstract class FirstPersonHandsMixin {
    @Inject(method={"submitHandsWithItems"}, at={@At(value="HEAD")}, cancellable=true)
    private void fpvdrone$hideHands(CallbackInfo ci) {
        if (DroneState.active) {
            ci.cancel();
        }
    }
}
