/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.screen;

import com.taylorgoolsby.fpvdrone.DroneController;

static class RatesSettingsScreen.1 {
    static final /* synthetic */ int[] $SwitchMap$com$taylorgoolsby$fpvdrone$DroneController$Axis;

    static {
        $SwitchMap$com$taylorgoolsby$fpvdrone$DroneController$Axis = new int[DroneController.Axis.values().length];
        try {
            RatesSettingsScreen.1.$SwitchMap$com$taylorgoolsby$fpvdrone$DroneController$Axis[DroneController.Axis.YAW.ordinal()] = 1;
        }
        catch (NoSuchFieldError noSuchFieldError) {
            // empty catch block
        }
        try {
            RatesSettingsScreen.1.$SwitchMap$com$taylorgoolsby$fpvdrone$DroneController$Axis[DroneController.Axis.PITCH.ordinal()] = 2;
        }
        catch (NoSuchFieldError noSuchFieldError) {
            // empty catch block
        }
    }
}
