/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.network.RegistryFriendlyByteBuf
 *  net.minecraft.network.codec.StreamCodec
 *  net.minecraft.server.level.ServerPlayer
 *  net.minecraftforge.event.network.CustomPayloadEvent$Context
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneDebug;
import com.taylorgoolsby.fpvdrone.DroneNetwork;
import com.taylorgoolsby.fpvdrone.DroneRegistry;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.server.level.ServerPlayer;
import net.minecraftforge.event.network.CustomPayloadEvent;

public record DroneOrientationPacket(float x, float y, float z, float w) {
    public static final StreamCodec<RegistryFriendlyByteBuf, DroneOrientationPacket> CODEC = StreamCodec.of((buf, msg) -> {
        buf.writeFloat(msg.x());
        buf.writeFloat(msg.y());
        buf.writeFloat(msg.z());
        buf.writeFloat(msg.w());
    }, buf -> new DroneOrientationPacket(buf.readFloat(), buf.readFloat(), buf.readFloat(), buf.readFloat()));

    public static void handle(DroneOrientationPacket msg, CustomPayloadEvent.Context ctx) {
        ctx.enqueueWork(() -> {
            ServerPlayer sender = ctx.getSender();
            if (sender == null) {
                return;
            }
            if (!DroneRegistry.isDrone(sender.getUUID())) {
                DroneDebug.log("server-reject-" + String.valueOf(sender.getUUID()), "server: orientation from " + sender.getName().getString() + " ignored, not a drone on the server");
                return;
            }
            float lengthSquared = msg.x() * msg.x() + msg.y() * msg.y() + msg.z() * msg.z() + msg.w() * msg.w();
            if (!Float.isFinite(lengthSquared) || lengthSquared < 1.0E-6f) {
                return;
            }
            DroneDebug.log("server-relay-" + String.valueOf(sender.getUUID()), "server: relaying orientation from " + sender.getName().getString() + " (" + msg.x() + ", " + msg.y() + ", " + msg.z() + ", " + msg.w() + ")");
            DroneNetwork.relayOrientation(sender, msg.x(), msg.y(), msg.z(), msg.w());
        });
        ctx.setPacketHandled(true);
    }
}
