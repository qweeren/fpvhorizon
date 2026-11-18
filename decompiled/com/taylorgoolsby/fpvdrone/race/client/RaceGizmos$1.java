/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.blaze3d.framegraph.FramePass
 *  net.minecraft.client.renderer.LevelTargetBundle
 *  net.minecraft.client.renderer.state.level.LevelRenderState
 *  net.minecraftforge.client.FramePassManager$PassDefinition
 */
package com.taylorgoolsby.fpvdrone.race.client;

import com.mojang.blaze3d.framegraph.FramePass;
import com.taylorgoolsby.fpvdrone.race.client.RaceGizmos;
import net.minecraft.client.renderer.LevelTargetBundle;
import net.minecraft.client.renderer.state.level.LevelRenderState;
import net.minecraftforge.client.FramePassManager;

static class RaceGizmos.1
implements FramePassManager.PassDefinition {
    RaceGizmos.1() {
    }

    public void extracts(LevelTargetBundle bundle, FramePass pass, LevelRenderState state) {
        bundle.main = pass.readsAndWrites(bundle.main);
    }

    public void executes(LevelRenderState state) {
        RaceGizmos.drawBuildGates();
    }
}
