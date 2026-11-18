/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.ChatFormatting
 *  net.minecraft.client.Options
 *  net.minecraft.client.gui.GuiGraphicsExtractor
 *  net.minecraft.client.gui.components.AbstractSliderButton
 *  net.minecraft.client.gui.components.Button
 *  net.minecraft.client.gui.components.events.GuiEventListener
 *  net.minecraft.client.gui.screens.Screen
 *  net.minecraft.client.gui.screens.options.OptionsSubScreen
 *  net.minecraft.network.chat.Component
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneController;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.function.Consumer;
import java.util.function.Supplier;
import net.minecraft.ChatFormatting;
import net.minecraft.client.Options;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsSubScreen;
import net.minecraft.network.chat.Component;

public class RatesSettingsScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.rates.title");
    private static final DroneController.Axis[] AXES = new DroneController.Axis[]{DroneController.Axis.YAW, DroneController.Axis.PITCH, DroneController.Axis.ROLL};
    private static final int COLOR_YAW = -43691;
    private static final int COLOR_PITCH = -11141291;
    private static final int COLOR_ROLL = -11184641;
    private static final int COLOR_FRAME = -1996488705;
    private static final int COLOR_GRID = 0x33FFFFFF;
    private static final int CURVE_THICKNESS = 2;
    private static final int HEADER = 36;
    private static final int FOOTER = 36;
    private final List<RateSlider> sliders = new ArrayList<RateSlider>();
    private Button infoLabel;
    private Button resetButton;
    private int graphX;
    private int graphY;
    private int graphW;
    private int graphH;

    public RatesSettingsScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
    }

    protected void init() {
        super.init();
        if (this.list != null) {
            this.removeWidget((GuiEventListener)this.list);
        }
        this.sliders.clear();
        for (DroneController.Axis a : AXES) {
            this.sliders.add((RateSlider)this.addRenderableWidget((GuiEventListener)new RateSlider(a, "rate", 0.01f, 2.55f, () -> Float.valueOf(DroneController.getRcRate(a)), v -> DroneController.setRcRate(a, v.floatValue()), this::updateInfo)));
            this.sliders.add((RateSlider)this.addRenderableWidget((GuiEventListener)new RateSlider(a, "super", 0.0f, 0.99f, () -> Float.valueOf(DroneController.getSuperRate(a)), v -> DroneController.setSuperRate(a, v.floatValue()), this::updateInfo)));
            this.sliders.add((RateSlider)this.addRenderableWidget((GuiEventListener)new RateSlider(a, "expo", 0.0f, 1.0f, () -> Float.valueOf(DroneController.getExpo(a)), v -> DroneController.setExpo(a, v.floatValue()), this::updateInfo)));
        }
        this.infoLabel = (Button)this.addRenderableWidget((GuiEventListener)Button.builder((Component)Component.empty(), b -> {}).build());
        this.infoLabel.active = false;
        this.resetButton = (Button)this.addRenderableWidget((GuiEventListener)Button.builder((Component)Component.translatable((String)"fpvdrone.rates.reset"), b -> {
            DroneController.resetRates();
            this.minecraft.gui.setScreen((Screen)new RatesSettingsScreen(this.lastScreen, this.options));
        }).build());
        this.layoutCustom();
        this.updateInfo();
    }

    protected void repositionElements() {
        super.repositionElements();
        this.layoutCustom();
    }

    private void layoutCustom() {
        if (this.sliders.isEmpty() || this.infoLabel == null || this.resetButton == null) {
            return;
        }
        int top = 36;
        int avail = this.height - 36 - 36;
        int half = this.width / 2;
        int sliderW = Math.min(200, half - 30);
        int sliderX = Math.max(10, (half - sliderW) / 2);
        int sliderH = Math.max(12, Math.min(20, (avail - 28) / 9));
        int y = top;
        for (int i = 0; i < this.sliders.size(); ++i) {
            RateSlider s = this.sliders.get(i);
            s.setX(sliderX);
            s.setY(y);
            s.setWidth(sliderW);
            s.setHeight(sliderH);
            y += sliderH + 2;
            if (i % 3 != 2) continue;
            y += 6;
        }
        int rightX = half + 15;
        int rightW = Math.max(60, this.width - rightX - 15);
        this.graphX = rightX;
        this.graphY = top;
        this.graphW = rightW;
        this.graphH = Math.max(40, Math.min(rightW, avail - 52));
        int labelY = this.graphY + this.graphH + 4;
        this.infoLabel.setX(rightX);
        this.infoLabel.setY(labelY);
        this.infoLabel.setWidth(rightW);
        this.infoLabel.setHeight(20);
        this.resetButton.setX(rightX);
        this.resetButton.setY(labelY + 24);
        this.resetButton.setWidth(rightW);
        this.resetButton.setHeight(20);
    }

    private void updateInfo() {
        if (this.infoLabel == null) {
            return;
        }
        this.infoLabel.setMessage((Component)Component.translatable((String)"fpvdrone.rates.full_stick", (Object[])new Object[]{RatesSettingsScreen.coloredRate(DroneController.Axis.YAW, ChatFormatting.RED), RatesSettingsScreen.coloredRate(DroneController.Axis.PITCH, ChatFormatting.GREEN), RatesSettingsScreen.coloredRate(DroneController.Axis.ROLL, ChatFormatting.BLUE)}));
    }

    private static Component coloredRate(DroneController.Axis a, ChatFormatting color) {
        return Component.literal((String)String.format(Locale.ROOT, "%.0f", Float.valueOf(DroneController.rateDegPerSec(a, 1.0f)))).withStyle(color);
    }

    public void extractRenderState(GuiGraphicsExtractor gg, int mouseX, int mouseY, float partialTick) {
        super.extractRenderState(gg, mouseX, mouseY, partialTick);
        this.drawGraph(gg);
    }

    private void drawGraph(GuiGraphicsExtractor gg) {
        if (this.graphW <= 0 || this.graphH <= 0) {
            return;
        }
        int right = this.graphX + this.graphW;
        int bottom = this.graphY + this.graphH;
        float max = 1.0f;
        for (DroneController.Axis a : AXES) {
            max = Math.max(max, DroneController.rateDegPerSec(a, 1.0f));
        }
        float graphMax = Math.max(100.0f, (float)Math.ceil(max / 100.0f) * 100.0f);
        for (int i = 1; i < 4; ++i) {
            int gy = this.graphY + this.graphH * i / 4;
            int gx = this.graphX + this.graphW * i / 4;
            gg.fill(this.graphX, gy, right, gy + 1, 0x33FFFFFF);
            gg.fill(gx, this.graphY, gx + 1, bottom, 0x33FFFFFF);
        }
        gg.fill(this.graphX, this.graphY, right, this.graphY + 1, -1996488705);
        gg.fill(this.graphX, bottom - 1, right, bottom, -1996488705);
        gg.fill(this.graphX, this.graphY, this.graphX + 1, bottom, -1996488705);
        gg.fill(right - 1, this.graphY, right, bottom, -1996488705);
        this.drawCurve(gg, DroneController.Axis.YAW, -43691, graphMax);
        this.drawCurve(gg, DroneController.Axis.PITCH, -11141291, graphMax);
        this.drawCurve(gg, DroneController.Axis.ROLL, -11184641, graphMax);
    }

    private void drawCurve(GuiGraphicsExtractor gg, DroneController.Axis axis, int color, float graphMax) {
        int bottom = this.graphY + this.graphH;
        int prevY = -1;
        for (int c = 0; c < this.graphW; ++c) {
            float stick = this.graphW > 1 ? (float)c / (float)(this.graphW - 1) : 0.0f;
            float deg = DroneController.rateDegPerSec(axis, stick);
            float t = Math.max(0.0f, Math.min(1.0f, deg / graphMax));
            int y = bottom - 1 - Math.round(t * (float)(this.graphH - 1));
            int yTop = prevY < 0 ? y : Math.min(prevY, y);
            int yBottom = prevY < 0 ? y : Math.max(prevY, y);
            gg.fill(this.graphX + c, yTop, this.graphX + c + 1, Math.min(bottom, yBottom + 2), color);
            prevY = y;
        }
    }

    public void removed() {
        DroneController.saveConfig();
        super.removed();
    }

    private static class RateSlider
    extends AbstractSliderButton {
        private final DroneController.Axis axis;
        private final String param;
        private final float min;
        private final float max;
        private final Supplier<Float> getter;
        private final Consumer<Float> setter;
        private final Runnable onChange;

        RateSlider(DroneController.Axis axis, String param, float min, float max, Supplier<Float> getter, Consumer<Float> setter, Runnable onChange) {
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
            this.setMessage((Component)Component.translatable((String)"fpvdrone.rates.slider", (Object[])new Object[]{Component.translatable((String)("fpvdrone.axis." + this.axis.key)), Component.translatable((String)("fpvdrone.rates.param." + this.param)), String.format(Locale.ROOT, "%.2f", this.getter.get())}).withStyle(RateSlider.axisColor(this.axis)));
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
}
