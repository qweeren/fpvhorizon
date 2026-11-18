/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.RaceGateException;
import java.util.ArrayList;
import java.util.function.BiPredicate;
import net.minecraft.core.BlockPos;

public class RaceTrack {
    public String raceTrackId;
    public String ownerUserId;
    public String name;
    public String dimension;
    public int startPosX;
    public int startPosY;
    public int startPosZ;
    public boolean deleted;
    public long dateUpdated;
    public long dateCreated;
    public ArrayList<RaceGate> gates;

    public BlockPos getStartPos() {
        return new BlockPos(this.startPosX, this.startPosY, this.startPosZ);
    }

    public void computeGates(BiPredicate<String, BlockPos> checkSolid) throws RaceGateException {
        for (RaceGate gate : this.gates) {
            try {
                gate.computeBoundaries(checkSolid);
            }
            catch (RaceGateException e) {
                String where = gate.a == null ? "" : "\nCheck near [" + gate.a.getX() + ", " + gate.a.getY() + ", " + gate.a.getZ() + "].";
                throw new RaceGateException(e.getMessage(), "The track has broken gates. " + e.endUserReason + where);
            }
        }
    }

    public int hashCode() {
        return this.raceTrackId.hashCode();
    }

    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof RaceTrack)) {
            return false;
        }
        RaceTrack t = (RaceTrack)o;
        return this.raceTrackId.equals(t.raceTrackId);
    }
}
