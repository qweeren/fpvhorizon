/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.V3;
import net.minecraft.core.BlockPos;

public final class GateCrossing {
    private static final double FLT_EPSILON = 1.1920929E-7;

    private GateCrossing() {
    }

    public static boolean passedThrough(RaceGate gate, String dimension, double[] prev, double[] cur, double[] intersection) {
        boolean gateHit;
        if (!dimension.equals(gate.dimension)) {
            return false;
        }
        double dx = cur[0] - prev[0];
        double dy = cur[1] - prev[1];
        double dz = cur[2] - prev[2];
        double travel2 = dx * dx + dy * dy + dz * dz;
        if (travel2 == 0.0) {
            return false;
        }
        double len = Math.sqrt(travel2);
        double ux = dx / len;
        double uy = dy / len;
        double uz = dz / len;
        double nx = gate.normal.x;
        double ny = gate.normal.y;
        double nz = gate.normal.z;
        double px = (double)gate.origin.getX() + Math.abs(nx) * 0.5;
        double py = (double)gate.origin.getY() + Math.abs(ny) * 0.5;
        double pz = (double)gate.origin.getZ() + Math.abs(nz) * 0.5;
        double constant = nx * px + ny * py + nz * pz;
        double denominator = nx * ux + ny * uy + nz * uz;
        if (denominator > -1.1920929E-7 && denominator < 1.1920929E-7) {
            return false;
        }
        double numerator = -(nx * prev[0] + ny * prev[1] + nz * prev[2] - constant);
        double ratio = numerator / denominator;
        if (ratio < 1.1920929E-7) {
            return false;
        }
        boolean bl = gateHit = travel2 - ratio * ratio > -1.0E-6;
        if (!gateHit) {
            return false;
        }
        double hx = prev[0] + ux * ratio;
        double hy = prev[1] + uy * ratio;
        double hz = prev[2] + uz * ratio;
        BlockPos crossingBlock = new BlockPos((int)Math.floor(hx), (int)Math.floor(hy), (int)Math.floor(hz));
        boolean pass = GateCrossing.isGateBlock(dimension, crossingBlock, gate);
        intersection[0] = hx;
        intersection[1] = hy;
        intersection[2] = hz;
        return pass;
    }

    public static boolean isGateBlock(String dimension, BlockPos pos, RaceGate gate) {
        boolean insideZ;
        boolean insideY;
        boolean insideX;
        if (!gate.dimension.equals(dimension)) {
            return false;
        }
        int minX = gate.origin.getX();
        int minY = gate.origin.getY();
        int minZ = gate.origin.getZ();
        int maxX = gate.farthest.getX();
        int maxY = gate.farthest.getY();
        int maxZ = gate.farthest.getZ();
        boolean bl = minX == maxX ? minX == pos.getX() : (insideX = minX <= pos.getX() && pos.getX() <= maxX);
        boolean bl2 = minY == maxY ? minY == pos.getY() : (insideY = minY <= pos.getY() && pos.getY() <= maxY);
        boolean bl3 = minZ == maxZ ? minZ == pos.getZ() : (insideZ = minZ <= pos.getZ() && pos.getZ() <= maxZ);
        if (!(insideX && insideY && insideZ)) {
            return false;
        }
        V3 originToCrossing = new V3(pos.getX() - minX, pos.getY() - minY, pos.getZ() - minZ);
        int row = (int)Math.floor(originToCrossing.dot(gate.up));
        int column = (int)Math.floor(originToCrossing.dot(gate.right));
        if (row < 0 || row >= gate.rowMin.length) {
            return false;
        }
        int rowMin = gate.rowMin[row];
        int rowMax = gate.rowMax[row];
        return rowMin <= column && column <= rowMax;
    }
}
