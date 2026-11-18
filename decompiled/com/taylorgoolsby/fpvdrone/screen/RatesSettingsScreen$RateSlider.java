/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.ChatFormatting
 *  net.minecraft.client.gui.components.AbstractSliderButton
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneController;
import java.util.Locale;
import java.util.function.Consumer;
import java.util.function.Supplier;
import net.minecraft.ChatFormatting;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.network.chat.Component;

private static class RatesSettingsScreen.RateSlider
extends AbstractSliderButton {
    private final DroneController.Axis axis;
    private final String param;
    private final float min;
    private final float max;
    private final Supplier<Float> getter;
    private final Consumer<Float> setter;
    private final Runnable onChange;

    RatesSettingsScreen.RateSlider(DroneController.Axis axis, String param, float min, float max, Supplier<Float> getter, Consumer<Float> setter, Runnable onChange) {
        super(0, 0, 150, 20, (Component)Component.empty(), (double)((getter.get().floatValue() - min) / (max - min)));
        this.axis = axis;
        this.param = param;
        this.min = min;
        this.max = max;
        this.getter = getter;
        this.setter = setter;
        this.onChange = onChange;
        this.updateMessage();
    }

    protected void updateMessage() {
        if (this.getter == null) {
            return;
        }
        this.setMessage((Component)Component.translatable((String)"fpvdrone.rates.slider", (Object[])new Object[]{Component.translatable((String)("fpvdrone.axis." + this.axis.key)), Component.translatable((String)("fpvdrone.rates.param." + this.param)), String.format(Locale.ROOT, "%.2f", this.getter.get())}).withStyle(RatesSettingsScreen.RateSlider.axisColor(this.axis)));
    }

    protected void applyValue() {
        float v = this.min + (float)this.value * (this.max - this.min);
        this.setter.accept(Float.valueOf((float)Math.round(v * 100.0f) / 100.0f));
        this.onChange.run();
    }

    private static ChatFormatting axisColor(DroneController.Axis a) {
        return switch (a) {
            case DroneController.Axis.YAW -> ChatFormatting.RED;
            case DroneController.Axis.PITCH -> ChatFormatting.GREEN;
            default -> ChatFormatting.BLUE;
        };
    }
}
