/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Options
 *  net.minecraft.client.gui.components.AbstractWidget
 *  net.minecraft.client.gui.components.Button
 *  net.minecraft.client.gui.screens.Screen
 *  net.minecraft.client.gui.screens.options.OptionsSubScreen
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneController;
import com.taylorgoolsby.fpvdrone.screen.BuildSettingsScreen;
import com.taylorgoolsby.fpvdrone.screen.ControllerBindingsScreen;
import com.taylorgoolsby.fpvdrone.screen.ControllerSelectScreen;
import com.taylorgoolsby.fpvdrone.screen.OtherSettingsScreen;
import com.taylorgoolsby.fpvdrone.screen.PhysicsSettingsScreen;
import com.taylorgoolsby.fpvdrone.screen.RatesSettingsScreen;
import net.minecraft.client.Options;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsSubScreen;
import net.minecraft.network.chat.Component;

public class FpvSettingsScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.settings.title");

    public FpvSettingsScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
        Button controller = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.controller"), b -> this.minecraft.gui.setScreen((Screen)new ControllerSelectScreen((Screen)this, this.options))).build();
        Button bindings = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.bindings"), b -> this.minecraft.gui.setScreen((Screen)new ControllerBindingsScreen((Screen)this, this.options))).build();
        bindings.active = DroneController.isConnected();
        Button rates = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.rates"), b -> this.minecraft.gui.setScreen((Screen)new RatesSettingsScreen((Screen)this, this.options))).build();
        Button build = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.build"), b -> this.minecraft.gui.setScreen((Screen)new BuildSettingsScreen((Screen)this, this.options))).build();
        Button physics = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.physics"), b -> this.minecraft.gui.setScreen((Screen)new PhysicsSettingsScreen((Screen)this, this.options))).build();
        Button other = Button.builder((Component)Component.translatable((String)"fpvdrone.settings.other"), b -> this.minecraft.gui.setScreen((Screen)new OtherSettingsScreen((Screen)this, this.options))).build();
        this.list.addSmall((AbstractWidget)controller, (AbstractWidget)bindings);
        this.list.addSmall((AbstractWidget)build, (AbstractWidget)rates);
        this.list.addSmall((AbstractWidget)physics, (AbstractWidget)other);
    }
}
