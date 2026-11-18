/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.blaze3d.framegraph.FramePass
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.renderer.LevelTargetBundle
 *  net.minecraft.client.renderer.state.level.LevelRenderState
 *  net.minecraft.gizmos.GizmoStyle
 *  net.minecraft.gizmos.Gizmos
 *  net.minecraft.resources.Identifier
 *  net.minecraft.util.ARGB
 *  net.minecraft.world.phys.AABB
 *  net.minecraftforge.client.FramePassManager$PassDefinition
 *  net.minecraftforge.client.event.AddFramePassEvent
 */
package com.taylorgoolsby.fpvdrone.race.client;

import com.mojang.blaze3d.framegraph.FramePass;
import com.taylorgoolsby.fpvdrone.race.RaceClientState;
import com.taylorgoolsby.fpvdrone.race.net.GateBox;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.LevelTargetBundle;
import net.minecraft.client.renderer.state.level.LevelRenderState;
import net.minecraft.gizmos.GizmoStyle;
import net.minecraft.gizmos.Gizmos;
import net.minecraft.resources.Identifier;
import net.minecraft.util.ARGB;
import net.minecraft.world.phys.AABB;
import net.minecraftforge.client.FramePassManager;
import net.minecraftforge.client.event.AddFramePassEvent;

final class RaceGizmos {
    private static final Identifier PASS_ID = Identifier.fromNamespaceAndPath((String)"fpvdrone", (String)"build_gates");
    private static final int WHITE = ARGB.colorFromFloat((float)1.0f, (float)1.0f, (float)1.0f, (float)1.0f);

    private RaceGizmos() {
    }

    static void register() {
        AddFramePassEvent.BUS.addListener(RaceGizmos::onAddFramePass);
    }

    private static void onAddFramePass(AddFramePassEvent event) {
        event.addPass(PASS_ID, new FramePassManager.PassDefinition(){

            public void extracts(LevelTargetBundle bundle, FramePass pass, LevelRenderState state) {
                bundle.main = pass.readsAndWrites(bundle.main);
            }

            public void executes(LevelRenderState state) {
                RaceGizmos.drawBuildGates();
            }
        });
    }

    private static void drawBuildGates() {
        if (!RaceClientState.buildMode) {
            return;
        }
        List<GateBox> gates = RaceClientState.buildGates;
        if (gates.isEmpty()) {
            return;
        }
        Minecraft mc = Minecraft.getInstance();
        if (mc.level == null) {
            return;
        }
        if (!RaceClientState.buildDimension.equals(mc.level.dimension().identifier().toString())) {
            return;
        }
        GizmoStyle style = GizmoStyle.stroke((int)WHITE);
        for (GateBox g : gates) {
            AABB box = new AABB((double)g.minX(), (double)g.minY(), (double)g.minZ(), (double)(g.maxX() + 1), (double)(g.maxY() + 1), (double)(g.maxZ() + 1));
            Gizmos.cuboid((AABB)box, (GizmoStyle)style, (boolean)true);
        }
    }
}
