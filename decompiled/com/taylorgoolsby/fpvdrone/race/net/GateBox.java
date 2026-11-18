/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.FriendlyByteBuf
 */
package com.taylorgoolsby.fpvdrone.race.net;

import net.minecraft.network.FriendlyByteBuf;

public record GateBox(int minX, int minY, int minZ, int maxX, int maxY, int maxZ) {
    public void write(FriendlyByteBuf buf) {
        buf.writeInt(this.minX);
        buf.writeInt(this.minY);
        buf.writeInt(this.minZ);
        buf.writeInt(this.maxX);
        buf.writeInt(this.maxY);
        buf.writeInt(this.maxZ);
    }

    public static GateBox read(FriendlyByteBuf buf) {
        return new GateBox(buf.readInt(), buf.readInt(), buf.readInt(), buf.readInt(), buf.readInt(), buf.readInt());
    }

    public double centerX() {
        return (double)(this.minX + this.maxX + 1) / 2.0;
    }

    public double centerY() {
        return (double)(this.minY + this.maxY + 1) / 2.0;
    }

    public double centerZ() {
        return (double)(this.minZ + this.maxZ + 1) / 2.0;
    }
}
