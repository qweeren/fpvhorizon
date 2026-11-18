/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Options
 *  net.minecraft.client.gui.components.AbstractSliderButton
 *  net.minecraft.client.gui.components.AbstractWidget
 *  net.minecraft.client.gui.components.Button
 *  net.minecraft.client.gui.screens.Screen
 *  net.minecraft.client.gui.screens.options.OptionsSubScreen
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.PhysicsSettings;
import net.minecraft.client.Options;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsSubScreen;
import net.minecraft.network.chat.Component;

public class PhysicsSettingsScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.physics.title");

    public PhysicsSettingsScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
        Button reset = Button.builder((Component)Component.translatable((String)"fpvdrone.physics.reset"), b -> {
            PhysicsSettings.reset();
            this.minecraft.gui.setScreen((Screen)new PhysicsSettingsScreen(this.lastScreen, this.options));
        }).build();
        this.list.addSmall((AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.AIR_DENSITY), (AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.DRONE_DENSITY));
        this.list.addSmall((AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.DRAG_COEFFICIENT), (AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.RESISTANCE_PER_1000_FT));
        this.list.addSmall((AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.TEMP_COEFF), (AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.SPECIFIC_HEAT));
        this.list.addSmall((AbstractWidget)PhysicsSettingsScreen.slider(PhysicsSettings.Param.HEAT_TRANSFER), (AbstractWidget)reset);
    }

    private static PhysicsSlider slider(PhysicsSettings.Param param) {
        return new PhysicsSlider(param);
    }

    public void removed() {
        PhysicsSettings.save();
        super.removed();
    }

    private static class PhysicsSlider
    extends AbstractSliderButton {
        private final PhysicsSettings.Param param;

        PhysicsSlider(PhysicsSettings.Param param) {
            super(0, 0, 150, 20, (Component)Component.empty(), (double)((PhysicsSettings.get(param) - param.min) / (param.max - param.min)));
            this.param = param;
            this.updateMessage();
        }

        protected void updateMessage() {
            if (this.param == null) {
                return;
            }
            this.setMessage((Component)Component.translatable((String)"fpvdrone.physics.slider", (Object[])new Object[]{Component.translatable((String)("fpvdrone.physics.param." + this.param.key)), this.param.format(PhysicsSettings.get(this.param))}));
        }

        protected void applyValue() {
            PhysicsSettings.set(this.param, this.param.min + (float)this.value * (this.param.max - this.param.min));
            this.value = (PhysicsSettings.get(this.param) - this.param.min) / (this.param.max - this.param.min);
        }
    }
}
