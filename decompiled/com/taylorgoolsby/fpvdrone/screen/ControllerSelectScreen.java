/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.ChatFormatting
 *  net.minecraft.client.Options
 *  net.minecraft.client.gui.components.AbstractWidget
 *  net.minecraft.client.gui.components.Button
 *  net.minecraft.client.gui.components.Button$OnPress
 *  net.minecraft.client.gui.screens.Screen
 *  net.minecraft.client.gui.screens.options.OptionsSubScreen
 *  net.minecraft.network.chat.Component
 *  net.minecraft.network.chat.MutableComponent
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneController;
import java.util.List;
import net.minecraft.ChatFormatting;
import net.minecraft.client.Options;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsSubScreen;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.MutableComponent;

public class ControllerSelectScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.controller.title");
    private static final int FULL_WIDTH = 310;

    public ControllerSelectScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
        List<DroneController.Device> devices = DroneController.listDevices();
        String selected = DroneController.selectedId();
        String active = DroneController.connectedId();
        this.list.addSmall((AbstractWidget)ControllerSelectScreen.row((Component)Component.translatable((String)"fpvdrone.controller.refresh"), b -> this.reopen()), null);
        MutableComponent auto = Component.translatable((String)"fpvdrone.controller.auto");
        if (selected.isEmpty()) {
            auto.withStyle(ChatFormatting.GREEN);
        }
        this.list.addSmall((AbstractWidget)ControllerSelectScreen.row((Component)auto, b -> this.choose("")), null);
        if (devices.isEmpty()) {
            Button none = ControllerSelectScreen.row((Component)Component.translatable((String)"fpvdrone.controller.none"), b -> {});
            none.active = false;
            this.list.addSmall((AbstractWidget)none, null);
            return;
        }
        for (DroneController.Device d : devices) {
            MutableComponent label = Component.literal((String)d.name()).append((Component)Component.translatable((String)(d.gamepad() ? "fpvdrone.controller.kind.gamepad" : "fpvdrone.controller.kind.raw")));
            if (d.id().equals(active)) {
                label.withStyle(ChatFormatting.GREEN);
            }
            this.list.addSmall((AbstractWidget)ControllerSelectScreen.row((Component)label, b -> this.choose(d.id())), null);
        }
    }

    private void choose(String id) {
        DroneController.selectDevice(id, this.minecraft);
        this.reopen();
    }

    private void reopen() {
        this.minecraft.gui.setScreen((Screen)new ControllerSelectScreen(this.lastScreen, this.options));
    }

    private static Button row(Component text, Button.OnPress onPress) {
        return Button.builder((Component)text, (Button.OnPress)onPress).width(310).build();
    }
}
