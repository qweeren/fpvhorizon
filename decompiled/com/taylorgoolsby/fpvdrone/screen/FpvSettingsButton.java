/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Minecraft
 *  net.minecraft.client.gui.components.AbstractWidget
 *  net.minecraft.client.gui.components.Button
 *  net.minecraft.client.gui.components.OptionsList
 *  net.minecraft.client.gui.components.events.GuiEventListener
 *  net.minecraft.client.gui.screens.Screen
 *  net.minecraft.client.gui.screens.options.controls.ControlsScreen
 *  net.minecraft.network.chat.Component
 *  net.minecraftforge.client.event.ScreenEvent$Init$Post
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.screen.FpvSettingsScreen;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.OptionsList;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.controls.ControlsScreen;
import net.minecraft.network.chat.Component;
import net.minecraftforge.client.event.ScreenEvent;

public final class FpvSettingsButton {
    private FpvSettingsButton() {
    }

    public static void register() {
        ScreenEvent.Init.Post.BUS.addListener(FpvSettingsButton::onScreenInit);
    }

    private static void onScreenInit(ScreenEvent.Init.Post event) {
        Screen screen = event.getScreen();
        if (!(screen instanceof ControlsScreen)) {
            return;
        }
        ControlsScreen controls = (ControlsScreen)screen;
        OptionsList list = null;
        for (GuiEventListener listener : event.getListenersList()) {
            OptionsList found;
            if (!(listener instanceof OptionsList)) continue;
            list = found = (OptionsList)listener;
            break;
        }
        if (list == null) {
            return;
        }
        Button button = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.button"), b -> {
            Minecraft mc = Minecraft.getInstance();
            mc.gui.setScreen((Screen)new FpvSettingsScreen((Screen)controls, mc.options));
        }).build();
        list.addSmall((AbstractWidget)button, null);
    }
}
