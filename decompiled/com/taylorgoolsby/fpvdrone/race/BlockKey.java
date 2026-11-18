/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 */
package com.taylorgoolsby.fpvdrone.race;

import java.util.Objects;
import net.minecraft.core.BlockPos;

public final class BlockKey {
    public final String dimension;
    public final BlockPos pos;

    public BlockKey(String dimension, BlockPos pos) {
        this.dimension = dimension;
        this.pos = pos;
    }

    public int hashCode() {
        return Objects.hash(this.dimension, this.pos);
    }

    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof BlockKey)) {
            return false;
        }
        BlockKey k = (BlockKey)o;
        return this.dimension.equals(k.dimension) && this.pos.equals((Object)k.pos);
    }

    public String toString() {
        return this.dimension + ", " + String.valueOf(this.pos);
    }
}
