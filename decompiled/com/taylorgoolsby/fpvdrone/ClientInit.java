/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraftforge.client.event.ClientPlayerNetworkEvent$LoggingOut
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneClientEvents;
import com.taylorgoolsby.fpvdrone.DroneHeadRender;
import com.taylorgoolsby.fpvdrone.DroneState;
import com.taylorgoolsby.fpvdrone.race.client.RaceClient;
import com.taylorgoolsby.fpvdrone.screen.FpvSettingsButton;
import net.minecraftforge.client.event.ClientPlayerNetworkEvent;

final class ClientInit {
    private ClientInit() {
    }

    static void init() {
        DroneClientEvents.register();
        DroneHeadRender.register();
        FpvSettingsButton.register();
        RaceClient.register();
        ClientPlayerNetworkEvent.LoggingOut.BUS.addListener(ClientInit::onLoggingOut);
    }

    private static void onLoggingOut(ClientPlayerNetworkEvent.LoggingOut event) {
        DroneState.reset();
    }
}
