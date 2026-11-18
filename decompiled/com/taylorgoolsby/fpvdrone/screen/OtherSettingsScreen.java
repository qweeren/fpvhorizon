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

public class OtherSettingsScreen
extends OptionsSubScreen {
    private static final Component TITLE = Component.translatable((String)"fpvdrone.other.title");
    private static final int FULL_WIDTH = 310;

    public OtherSettingsScreen(Screen lastScreen, Options options) {
        super(lastScreen, options, TITLE);
    }

    protected void addOptions() {
        Button mode3d = Button.builder((Component)OtherSettingsScreen.mode3dLabel(), b -> {
            DroneController.setMode3d(!DroneController.isMode3d());
            b.setMessage(OtherSettingsScreen.mode3dLabel());
        }).width(310).build();
        Button showAxisValues = Button.builder((Component)OtherSettingsScreen.showAxisValuesLabel(), b -> {
            DroneController.setShowAxisValues(!DroneController.isShowAxisValues());
            b.setMessage(OtherSettingsScreen.showAxisValuesLabel());
        }).width(310).build();
        Button showHeadDebug = Button.builder((Component)OtherSettingsScreen.showHeadDebugLabel(), b -> {
            DroneController.setShowHeadDebug(!DroneController.isShowHeadDebug());
            b.setMessage(OtherSettingsScreen.showHeadDebugLabel());
        }).width(310).build();
        this.list.addSmall((AbstractWidget)mode3d, null);
        this.list.addSmall((AbstractWidget)showAxisValues, null);
        this.list.addSmall((AbstractWidget)showHeadDebug, null);
    }

    private static Component mode3dLabel() {
        return Component.translatable((String)"fpvdrone.other.mode_3d", (Object[])new Object[]{DroneController.isMode3d() ? CommonComponents.OPTION_ON : CommonComponents.OPTION_OFF});
    }

    private static Component showHeadDebugLabel() {
        return Component.translatable((String)"fpvdrone.other.show_head_debug", (Object[])new Object[]{DroneController.isShowHeadDebug() ? CommonComponents.OPTION_ON : CommonComponents.OPTION_OFF});
    }

    private static Component showAxisValuesLabel() {
        return Component.translatable((String)"fpvdrone.other.show_axis_values", (Object[])new Object[]{DroneController.isShowAxisValues() ? CommonComponents.OPTION_ON : CommonComponents.OPTION_OFF});
    }
}
