/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.RegistryFriendlyByteBuf
 *  net.minecraft.network.codec.StreamCodec
 *  net.minecraftforge.event.network.CustomPayloadEvent$Context
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneOrientations;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import java.util.UUID;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record DroneStatePacket(UUID player, boolean active) {
    public static final StreamCodec<RegistryFriendlyByteBuf, DroneStatePacket> CODEC = StreamCodec.of((buf, msg) -> {
        buf.writeUUID(msg.player());
        buf.writeBoolean(msg.active());
    }, buf -> new DroneStatePacket(buf.readUUID(), buf.readBoolean()));

    public static void handle(DroneStatePacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> {
            DroneRegistry.set(msg.player(), msg.active());
            if (!msg.active()) {
                DroneOrientations.remove(msg.player());
            }
        });
        ctx.setPacketHandled(true);
    }
}
