/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 *  net.minecraft.core.Direction
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.RaceGateException;
import com.taylorgoolsby.fpvdrone.race.RaceMath;
import com.taylorgoolsby.fpvdrone.race.V3;
import java.util.ArrayList;
import java.util.function.BiPredicate;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;

public class RaceGate {
    public String dimension;
    public BlockPos a;
    public Direction face;
    public BlockPos b;
    public String raceTrackId;
    public int index;
    public BlockPos origin;
    public BlockPos farthest;
    public int[] rowMin;
    public int[] rowMax;
    public V3 right;
    public V3 up;
    public V3 normal;
    public ArrayList<BlockPos> path;

    public RaceGate(String dimension, BlockPos a, Direction face, BlockPos b) {
        this.dimension = dimension;
        this.a = a;
        this.face = face;
        this.b = b;
    }

    public void computeBoundaries(BiPredicate<String, BlockPos> checkSolid) throws RaceGateException {
        if (this.dimension == null || this.a == null || this.face == null || this.b == null) {
            throw new RaceGateException("Gate has null values from DB.", "This track is not supported in this version. Please rebuild the track.");
        }
        RaceMath.loadGate(this.dimension, this.a, this.face, this.b, checkSolid, this);
    }

    public void setBoundaries(BlockPos origin, BlockPos farthest, int[] rowMin, int[] rowMax, V3 right, V3 up, V3 normal, ArrayList<BlockPos> path) {
        this.origin = origin;
        this.farthest = farthest;
        this.rowMin = rowMin;
        this.rowMax = rowMax;
        this.right = right;
        this.up = up;
        this.normal = normal;
        this.path = path;
    }

    public V3 getCenter() {
        return new V3((float)(this.origin.getX() + this.farthest.getX() + 1) / 2.0f, (float)(this.origin.getY() + this.farthest.getY() + 1) / 2.0f, (float)(this.origin.getZ() + this.farthest.getZ() + 1) / 2.0f);
    }

    public boolean containsBlock(BlockPos p) {
        return this.origin.getX() <= p.getX() && p.getX() <= this.farthest.getX() && this.origin.getY() <= p.getY() && p.getY() <= this.farthest.getY() && this.origin.getZ() <= p.getZ() && p.getZ() <= this.farthest.getZ();
    }
}
