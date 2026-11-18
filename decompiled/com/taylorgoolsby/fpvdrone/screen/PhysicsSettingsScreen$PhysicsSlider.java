/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.gui.components.AbstractSliderButton
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.PhysicsSettings;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.network.chat.Component;

private static class PhysicsSettingsScreen.PhysicsSlider
extends AbstractSliderButton {
    private final PhysicsSettings.Param param;

    PhysicsSettingsScreen.PhysicsSlider(PhysicsSettings.Param param) {
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
