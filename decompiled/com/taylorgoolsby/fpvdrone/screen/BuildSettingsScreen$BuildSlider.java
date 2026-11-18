/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.gui.components.AbstractSliderButton
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneBuild;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.network.chat.Component;

private static class BuildSettingsScreen.BuildSlider
extends AbstractSliderButton {
    private final DroneBuild.Param param;
    private final Runnable onChange;

    BuildSettingsScreen.BuildSlider(DroneBuild.Param param, Runnable onChange) {
        super(0, 0, 150, 20, (Component)Component.empty(), (double)((DroneBuild.get(param) - param.min) / (param.max - param.min)));
        this.param = param;
        this.onChange = onChange;
        this.updateMessage();
    }

    protected void updateMessage() {
        if (this.param == null) {
            return;
        }
        this.setMessage((Component)Component.translatable((String)"fpvdrone.build.slider", (Object[])new Object[]{Component.translatable((String)("fpvdrone.build.param." + this.param.key)), this.param.format(DroneBuild.get(this.param))}));
    }

    protected void applyValue() {
        DroneBuild.set(this.param, this.param.min + (float)this.value * (this.param.max - this.param.min));
        this.value = (DroneBuild.get(this.param) - this.param.min) / (this.param.max - this.param.min);
        this.onChange.run();
    }
}
