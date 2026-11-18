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

import com.taylorgoolsby.fpvdrone.DroneBuild;
import java.util.Locale;
import net.minecraft.client.Options;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.options.OptionsSubScreen;
import net.minecraft.network.chat.Component;

public class BuildSettingsScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.build.title");
    private Button bladeMassLabel;
    private Button motorMassLabel;

    public BuildSettingsScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
        this.bladeMassLabel = BuildSettingsScreen.inactiveLabel();
        this.motorMassLabel = BuildSettingsScreen.inactiveLabel();
        Button reset = Button.builder((Component)Component.translatable((String)"fpvdrone.build.reset"), b -> {
            DroneBuild.reset();
            this.minecraft.gui.setScreen((Screen)new BuildSettingsScreen(this.lastScreen, this.options));
        }).build();
        this.list.addSmall((AbstractWidget)this.slider(DroneBuild.Param.DRONE_MASS), (AbstractWidget)this.slider(DroneBuild.Param.BATTERY_CELLS));
        this.list.addSmall((AbstractWidget)this.slider(DroneBuild.Param.MOTOR_KV), (AbstractWidget)this.slider(DroneBuild.Param.BLADES));
        this.list.addSmall((AbstractWidget)this.slider(DroneBuild.Param.PROP_RADIUS), (AbstractWidget)this.slider(DroneBuild.Param.PROP_PITCH));
        this.list.addSmall((AbstractWidget)this.slider(DroneBuild.Param.PROP_WIDTH), (AbstractWidget)this.bladeMassLabel);
        this.list.addSmall((AbstractWidget)this.slider(DroneBuild.Param.MOTOR_WIDTH), (AbstractWidget)this.slider(DroneBuild.Param.MOTOR_HEIGHT));
        this.list.addSmall((AbstractWidget)this.motorMassLabel, (AbstractWidget)this.slider(DroneBuild.Param.CAMERA_ANGLE));
        this.list.addSmall((AbstractWidget)reset, null);
        this.updateDerivedLabels();
    }

    private BuildSlider slider(DroneBuild.Param param) {
        return new BuildSlider(param, this::updateDerivedLabels);
    }

    private static Button inactiveLabel() {
        Button label = Button.builder((Component)Component.empty(), b -> {}).build();
        label.active = false;
        return label;
    }

    private void updateDerivedLabels() {
        if (this.bladeMassLabel != null) {
            this.bladeMassLabel.setMessage((Component)Component.translatable((String)"fpvdrone.build.blade_mass", (Object[])new Object[]{BuildSettingsScreen.grams(DroneBuild.bladeMass())}));
        }
        if (this.motorMassLabel != null) {
            this.motorMassLabel.setMessage((Component)Component.translatable((String)"fpvdrone.build.motor_mass", (Object[])new Object[]{BuildSettingsScreen.grams(DroneBuild.motorMass())}));
        }
    }

    private static String grams(float kilograms) {
        return String.format(Locale.ROOT, "%.1f g", Float.valueOf(kilograms * 1000.0f));
    }

    public void removed() {
        DroneBuild.save();
        super.removed();
    }

    private static class BuildSlider
    extends AbstractSliderButton {
        private final DroneBuild.Param param;
        private final Runnable onChange;

        BuildSlider(DroneBuild.Param param, Runnable onChange) {
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
}
