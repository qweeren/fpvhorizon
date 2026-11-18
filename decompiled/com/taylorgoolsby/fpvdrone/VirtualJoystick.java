/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  com.mojang.blaze3d.platform.InputConstants
 *  org.lwjgl.sdl.SDLJoystick
 *  org.lwjgl.sdl.SDL_VirtualJoystickDesc
 *  org.lwjgl.system.MemoryStack
 */
package com.taylorgoolsby.fpvdrone;

import com.mojang.blaze3d.platform.InputConstants;
import com.taylorgoolsby.fpvdrone.FpvDroneMod;
import org.lwjgl.sdl.SDLJoystick;
import org.lwjgl.sdl.SDL_VirtualJoystickDesc;
import org.lwjgl.system.MemoryStack;

final class VirtualJoystick {
    static final boolean ENABLED = !"false".equalsIgnoreCase(System.getProperty("fpvdrone.virtual"));
    private static final String NAME = "FPV Virtual Joystick (WASD + Arrows)";
    private static final int KEY_RIGHT = 79;
    private static final int KEY_LEFT = 80;
    private static final int KEY_DOWN = 81;
    private static final int KEY_UP = 82;
    private static final int KEY_A = 4;
    private static final int KEY_D = 7;
    private static final int KEY_S = 22;
    private static final int KEY_W = 26;
    private static final int ROLL = 0;
    private static final int PITCH = 1;
    private static final int THROTTLE = 2;
    private static final int YAW = 3;
    private static final float RAMP = 0.25f;
    private static final float RETURN = 0.5f;
    private static final float THROTTLE_STEP = 0.04f;
    private static int instanceId = 0;
    private static long joystick = 0L;
    private static boolean failed = false;
    private static final float[] axes = new float[]{0.0f, 0.0f, -1.0f, 0.0f};

    private VirtualJoystick() {
    }

    static boolean isInstance(int id) {
        return instanceId != 0 && id == instanceId;
    }

    static void ensureAttached() {
        if (!ENABLED || failed || instanceId != 0) {
            return;
        }
        try (MemoryStack stack = MemoryStack.stackPush();){
            SDL_VirtualJoystickDesc desc = SDL_VirtualJoystickDesc.calloc((MemoryStack)stack);
            desc.version(SDL_VirtualJoystickDesc.SIZEOF).type((short)0).vendor_id((short)18000).product_id((short)22017).naxes((short)4).nbuttons((short)0).nhats((short)0).name(stack.UTF8((CharSequence)NAME));
            int id = SDLJoystick.SDL_AttachVirtualJoystick((SDL_VirtualJoystickDesc)desc);
            if (id == 0) {
                failed = true;
                FpvDroneMod.LOGGER.warn("[fpvdrone] could not create virtual joystick");
                return;
            }
            long j = SDLJoystick.SDL_OpenJoystick((int)id);
            if (j == 0L) {
                SDLJoystick.SDL_DetachVirtualJoystick((int)id);
                failed = true;
                FpvDroneMod.LOGGER.warn("[fpvdrone] could not open virtual joystick");
                return;
            }
            instanceId = id;
            joystick = j;
            VirtualJoystick.push();
            FpvDroneMod.LOGGER.info("[fpvdrone] virtual joystick active: WASD + arrow keys");
        }
    }

    static void tick() {
        if (!ENABLED) {
            return;
        }
        VirtualJoystick.ensureAttached();
        if (joystick == 0L) {
            return;
        }
        VirtualJoystick.axes[0] = VirtualJoystick.approach(axes[0], VirtualJoystick.down(79) - VirtualJoystick.down(80));
        VirtualJoystick.axes[1] = VirtualJoystick.approach(axes[1], VirtualJoystick.down(81) - VirtualJoystick.down(82));
        VirtualJoystick.axes[3] = VirtualJoystick.approach(axes[3], VirtualJoystick.down(7) - VirtualJoystick.down(4));
        VirtualJoystick.axes[2] = VirtualJoystick.clamp(axes[2] + (VirtualJoystick.down(26) - VirtualJoystick.down(22)) * 0.04f);
        VirtualJoystick.push();
    }

    private static void push() {
        for (int i = 0; i < axes.length; ++i) {
            SDLJoystick.SDL_SetJoystickVirtualAxis((long)joystick, (int)i, (short)((short)Math.round(VirtualJoystick.clamp(axes[i]) * 32767.0f)));
        }
    }

    private static float down(int key) {
        return InputConstants.isKeyDown((int)key) ? 1.0f : 0.0f;
    }

    private static float approach(float v, float target) {
        float step = target == 0.0f ? 0.5f : 0.25f;
        float d = target - v;
        return v + Math.max(-step, Math.min(step, d));
    }

    private static float clamp(float v) {
        return Math.max(-1.0f, Math.min(1.0f, v));
    }
}
