/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Options
 *  net.minecraft.client.gui.components.AbstractWidget
 *  net.minecraft.client.gui.components.Button
 *  net.minecraft.client.gui.screens.Screen
 *  net.minecraft.client.gui.screens.options.OptionsSubScreen
 *  net.minecraft.network.chat.CommonComponents
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneController;
import net.minecraft.client.Options;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsSubScreen;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;

public class ControllerBindingsScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.bindings.title");
    private static final int FULL_WIDTH = 310;
    private final Button[] bindButtons = new Button[DroneController.Axis.values().length];
    private final Button[] invertButtons = new Button[DroneController.Axis.values().length];
    private Button resetButton;

    public ControllerBindingsScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
        boolean connected = DroneController.isConnected();
        Button info = Button.builder((Component)(connected ? Component.translatable((String)"fpvdrone.bindings.controller", (Object[])new Object[]{DroneController.connectedName()}) : Component.translatable((String)"fpvdrone.bindings.none")), b -> {}).width(310).build();
        info.active = false;
        this.list.addSmall((AbstractWidget)info, null);
        for (DroneController.Axis a : DroneController.Axis.values()) {
            Button bind = Button.builder((Component)ControllerBindingsScreen.bindLabel(a), b -> this.onBindPressed(a)).build();
            Button invert = Button.builder((Component)ControllerBindingsScreen.invertLabel(a), b -> {
                DroneController.toggleInvert(a);
                b.setMessage(ControllerBindingsScreen.invertLabel(a));
            }).build();
            bind.active = connected;
            invert.active = connected;
            this.bindButtons[a.ordinal()] = bind;
            this.invertButtons[a.ordinal()] = invert;
            this.list.addSmall((AbstractWidget)bind, (AbstractWidget)invert);
        }
        this.resetButton = Button.builder((Component)Component.translatable((String)"fpvdrone.bindings.reset"), b -> {
            DroneController.resetBindings();
            this.reopen();
        }).width(310).build();
        this.resetButton.active = connected;
        this.list.addSmall((AbstractWidget)this.resetButton, null);
    }

    private void onBindPressed(DroneController.Axis a) {
        if (DroneController.capturingAxis() == a) {
            DroneController.cancelCapture();
            this.reopen();
            return;
        }
        boolean started = DroneController.beginCapture(a, () -> {
            if (this.minecraft.gui.screen() == this) {
                this.reopen();
            }
        });
        if (!started) {
            return;
        }
        for (DroneController.Axis other : DroneController.Axis.values()) {
            this.invertButtons[other.ordinal()].active = false;
            if (other == a) continue;
            this.bindButtons[other.ordinal()].active = false;
        }
        this.resetButton.active = false;
        this.bindButtons[a.ordinal()].setMessage((Component)Component.translatable((String)("fpvdrone.bindings.listening." + a.key)));
    }

    private static Component bindLabel(DroneController.Axis a) {
        return Component.translatable((String)"fpvdrone.bindings.entry", (Object[])new Object[]{Component.translatable((String)("fpvdrone.axis." + a.key)), DroneController.bindingName(a)});
    }

    private static Component invertLabel(DroneController.Axis a) {
        return Component.translatable((String)"fpvdrone.bindings.invert", (Object[])new Object[]{DroneController.isInverted(a) ? CommonComponents.OPTION_ON : CommonComponents.OPTION_OFF});
    }

    private void reopen() {
        this.minecraft.gui.setScreen((Screen)new ControllerBindingsScreen(this.lastScreen, this.options));
    }

    public void removed() {
        DroneController.cancelCapture();
        super.removed();
    }
}
