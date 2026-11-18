/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.world.entity.EntityDimensions
 *  net.minecraft.world.entity.LivingEntity
 *  net.minecraft.world.entity.Pose
 *  net.minecraft.world.entity.player.Player
 *  org.spongepowered.asm.mixin.Mixin
 *  org.spongepowered.asm.mixin.injection.At
 *  org.spongepowered.asm.mixin.injection.Inject
 *  org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable
 */
package com.taylorgoolsby.fpvdrone.mixin;

import com.taylorgoolsby.fpvdrone.DroneRegistry;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.player.Player;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value={LivingEntity.class})
public abstract class PlayerDimensionsMixin {
    private static final EntityDimensions FPV_DIMENSIONS = EntityDimensions.scalable((float)0.3f, (float)0.3f).withEyeHeight(0.15f);

    @Inject(method={"getDimensions"}, at={@At(value="RETURN")}, cancellable=true)
    private void fpvdrone$changeDimensions(Pose pose, CallbackInfoReturnable<EntityDimensions> cir) {
        Player p;
        PlayerDimensionsMixin playerDimensionsMixin = this;
        if (playerDimensionsMixin instanceof Player && DroneRegistry.isDrone((p = (Player)playerDimensionsMixin).getUUID())) {
            cir.setReturnValue((Object)FPV_DIMENSIONS);
        }
    }
}
