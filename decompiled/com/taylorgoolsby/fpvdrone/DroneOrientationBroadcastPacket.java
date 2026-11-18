/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.RegistryFriendlyByteBuf
 *  net.minecraft.network.codec.StreamCodec
 *  net.minecraftforge.event.network.CustomPayloadEvent$Context
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneDebug;
import com.taylorgoolsby.fpvdrone.DroneOrientations;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import java.util.UUID;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record DroneOrientationBroadcastPacket(UUID player, float x, float y, float z, float w) {
    public static final StreamCodec<RegistryFriendlyByteBuf, DroneOrientationBroadcastPacket> CODEC = StreamCodec.of((buf, msg) -> {
        buf.writeUUID(msg.player());
        buf.writeFloat(msg.x());
        buf.writeFloat(msg.y());
        buf.writeFloat(msg.z());
        buf.writeFloat(msg.w());
    }, buf -> new DroneOrientationBroadcastPacket(buf.readUUID(), buf.readFloat(), buf.readFloat(), buf.readFloat(), buf.readFloat()));

    public static void handle(DroneOrientationBroadcastPacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> {
            boolean drone = DroneRegistry.isDrone(msg.player());
            DroneDebug.markReceived(msg.player());
            DroneDebug.log("client-recv-" + String.valueOf(msg.player()), "client: orientation for " + String.valueOf(msg.player()) + " registryDrone=" + drone + " (" + msg.x() + ", " + msg.y() + ", " + msg.z() + ", " + msg.w() + ")");
            if (drone) {
                DroneOrientations.set(msg.player(), msg.x(), msg.y(), msg.z(), msg.w());
            }
        });
        ctx.setPacketHandled(true);
    }
}
