/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraftforge.client.event.ClientPlayerNetworkEvent$LoggingOut
 */
package com.taylorgoolsby.fpvdrone.race.client;

import com.taylorgoolsby.fpvdrone.race.RaceClientState;
import com.taylorgoolsby.fpvdrone.race.client.RaceGizmos;
import com.taylorgoolsby.fpvdrone.race.client.RaceHud;
import net.minecraftforge.client.event.ClientPlayerNetworkEvent;

public final class RaceClient {
    private RaceClient() {
    }

    public static void register() {
        RaceHud.register();
        RaceGizmos.register();
        ClientPlayerNetworkEvent.LoggingOut.BUS.addListener(RaceClient::onLoggingOut);
    }

    private static void onLoggingOut(ClientPlayerNetworkEvent.LoggingOut event) {
        RaceClientState.reset();
    }
}
