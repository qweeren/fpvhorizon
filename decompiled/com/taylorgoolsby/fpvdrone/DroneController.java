/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.client.Minecraft
 *  net.minecraft.network.chat.Component
 *  net.minecraftforge.fml.loading.FMLPaths
 *  org.lwjgl.sdl.SDLGamepad
 *  org.lwjgl.sdl.SDLInit
 *  org.lwjgl.sdl.SDLJoystick
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneDebug;
import com.taylorgoolsby.fpvdrone.DroneState;
import com.taylorgoolsby.fpvdrone.FpvDroneMod;
import com.taylorgoolsby.fpvdrone.VirtualJoystick;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.IntBuffer;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.OpenOption;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Properties;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;
import net.minecraftforge.fml.loading.FMLPaths;
import org.lwjgl.sdl.SDLGamepad;
import org.lwjgl.sdl.SDLInit;
import org.lwjgl.sdl.SDLJoystick;

public final class DroneController {
    public static float roll;
    public static float pitch;
    public static float yaw;
    public static float throttle;
    public static float rawRoll;
    public static float rawPitch;
    public static float rawYaw;
    public static float rawThrottle;
    public static float rollDegPerSec;
    public static float pitchDegPerSec;
    public static float yawDegPerSec;
    private static boolean showAxisValues;
    private static boolean showHeadDebug;
    private static boolean mode3d;
    private static final short[] debugRaw;
    private static final short[] debugMin;
    private static final short[] debugMax;
    private static int debugTicks;
    private static long handle;
    private static boolean isGamepad;
    private static String name;
    private static String connectedId;
    private static boolean virtual;
    private static int scanCooldown;
    private static boolean sdlReady;
    private static float deadzone;
    public static final float RC_RATE_MIN = 0.01f;
    public static final float RC_RATE_MAX = 2.55f;
    public static final float SUPER_RATE_MIN = 0.0f;
    public static final float SUPER_RATE_MAX = 0.99f;
    public static final float EXPO_MIN = 0.0f;
    public static final float EXPO_MAX = 1.0f;
    private static final float DEFAULT_RC_RATE = 0.67f;
    private static final float DEFAULT_SUPER_RATE = 0.2f;
    private static final float DEFAULT_EXPO = 0.67f;
    private static final float[] rcRate;
    private static final float[] superRate;
    private static final float[] expo;
    private static String selectedId;
    private static final int[] GP_AXIS_DEFAULT;
    private static final boolean[] GP_INVERT_DEFAULT;
    private static final int[] RAW_AXIS_DEFAULT;
    private static final boolean[] RAW_INVERT_DEFAULT;
    private static final int[] gpAxis;
    private static final boolean[] gpInvert;
    private static final int[] rawAxis;
    private static final boolean[] rawInvert;
    private static final int GAMEPAD_AXIS_COUNT = 6;
    private static final String[] GP_AXIS_NAME_KEYS;
    private static final float CAPTURE_THRESHOLD = 0.5f;
    private static Axis captureTarget;
    private static Runnable captureDone;
    private static float[] captureBase;
    private static boolean configLoaded;

    private DroneController() {
    }

    public static boolean isConnected() {
        return handle != 0L;
    }

    public static String connectedName() {
        return name;
    }

    public static boolean isVirtual() {
        return virtual;
    }

    public static String connectedId() {
        return connectedId;
    }

    public static String selectedId() {
        DroneController.ensureConfig();
        return selectedId;
    }

    public static List<Device> listDevices() {
        DroneController.ensureConfig();
        ArrayList<Device> out = new ArrayList<Device>();
        if (!DroneController.ensureSdl()) {
            return out;
        }
        VirtualJoystick.ensureAttached();
        SDLJoystick.SDL_UpdateJoysticks();
        IntBuffer ids = SDLJoystick.SDL_GetJoysticks();
        if (ids == null) {
            return out;
        }
        HashSet<String> seen = new HashSet<String>();
        for (int i = 0; i < ids.remaining(); ++i) {
            String key;
            int id = ids.get(ids.position() + i);
            boolean gp = SDLGamepad.SDL_IsGamepad((int)id);
            if (!gp && !DroneController.hasFlightAxes(id) || !seen.add(key = DroneController.deviceKey(id))) continue;
            out.add(new Device(key, DroneController.deviceName(id), gp));
        }
        return out;
    }

    public static void selectDevice(String id, Minecraft mc) {
        DroneController.ensureConfig();
        selectedId = id == null ? "" : id;
        DroneController.saveConfig();
        DroneController.cancelCapture();
        DroneController.disconnect();
        scanCooldown = 0;
        if (DroneController.ensureSdl()) {
            DroneController.scanAndConnect(mc);
        }
    }

    private static boolean hasFlightAxes(int instanceId) {
        long j = SDLJoystick.SDL_OpenJoystick((int)instanceId);
        if (j == 0L) {
            return false;
        }
        boolean ok = SDLJoystick.SDL_GetNumJoystickAxes((long)j) >= 4;
        SDLJoystick.SDL_CloseJoystick((long)j);
        return ok;
    }

    private static String deviceName(int instanceId) {
        String n = SDLJoystick.SDL_GetJoystickNameForID((int)instanceId);
        return n != null ? n : "Unknown controller";
    }

    private static String deviceKey(int instanceId) {
        int vid = SDLJoystick.SDL_GetJoystickVendorForID((int)instanceId) & 0xFFFF;
        int pid = SDLJoystick.SDL_GetJoystickProductForID((int)instanceId) & 0xFFFF;
        return String.format("%04x:%04x:%s", vid, pid, DroneController.deviceName(instanceId));
    }

    private static void ensureConfig() {
        if (!configLoaded) {
            DroneController.loadConfig();
        }
    }

    private static boolean ensureSdl() {
        if (!sdlReady) {
            sdlReady = SDLInit.SDL_InitSubSystem((int)8704);
        }
        return sdlReady;
    }

    public static void tick(Minecraft mc) {
        DroneController.ensureConfig();
        DroneController.showDebugReadout(mc);
        if (!DroneController.ensureSdl()) {
            return;
        }
        VirtualJoystick.tick();
        SDLJoystick.SDL_UpdateJoysticks();
        if (handle != 0L) {
            boolean connected;
            boolean bl = connected = isGamepad ? SDLGamepad.SDL_GamepadConnected((long)handle) : SDLJoystick.SDL_JoystickConnected((long)handle);
            if (!connected) {
                String lost = name;
                DroneController.cancelCapture();
                DroneController.disconnect();
                DroneController.notify(mc, "Controller disconnected: " + lost);
                return;
            }
            if (captureTarget != null) {
                DroneController.updateCapture();
            }
            return;
        }
        if (--scanCooldown > 0) {
            return;
        }
        scanCooldown = 20;
        DroneController.scanAndConnect(mc);
    }

    private static void scanAndConnect(Minecraft mc) {
        IntBuffer ids = SDLJoystick.SDL_GetJoysticks();
        if (ids == null) {
            return;
        }
        for (int i = 0; i < ids.remaining(); ++i) {
            int id = ids.get(ids.position() + i);
            String key = DroneController.deviceKey(id);
            if (!selectedId.isEmpty() && !selectedId.equals(key)) continue;
            if (SDLGamepad.SDL_IsGamepad((int)id)) {
                long g = SDLGamepad.SDL_OpenGamepad((int)id);
                if (g == 0L) continue;
                handle = g;
                isGamepad = true;
            } else {
                long j = SDLJoystick.SDL_OpenJoystick((int)id);
                if (j == 0L) continue;
                if (SDLJoystick.SDL_GetNumJoystickAxes((long)j) < 4) {
                    SDLJoystick.SDL_CloseJoystick((long)j);
                    continue;
                }
                handle = j;
                isGamepad = false;
            }
            DroneController.resetDebugRange();
            name = DroneController.deviceName(id);
            connectedId = key;
            virtual = VirtualJoystick.isInstance(id);
            DroneController.notify(mc, "Controller connected: " + name + (isGamepad ? " (gamepad)" : " (RC/raw)"));
            break;
        }
    }

    private static void disconnect() {
        if (handle != 0L) {
            if (isGamepad) {
                SDLGamepad.SDL_CloseGamepad((long)handle);
            } else {
                SDLJoystick.SDL_CloseJoystick((long)handle);
            }
        }
        handle = 0L;
        name = "";
        connectedId = "";
        virtual = false;
        throttle = 0.0f;
        yaw = 0.0f;
        pitch = 0.0f;
        roll = 0.0f;
        yawDegPerSec = 0.0f;
        pitchDegPerSec = 0.0f;
        rollDegPerSec = 0.0f;
        rawThrottle = 0.0f;
        rawYaw = 0.0f;
        rawPitch = 0.0f;
        rawRoll = 0.0f;
        DroneController.resetDebugRange();
        scanCooldown = 20;
    }

    private static void showDebugReadout(Minecraft mc) {
        boolean axes;
        boolean bl = axes = showAxisValues && DroneState.active && handle != 0L;
        if (!axes && !showHeadDebug) {
            return;
        }
        if (++debugTicks % 4 != 0) {
            return;
        }
        StringBuilder text = new StringBuilder();
        if (axes) {
            text.append(DroneController.debugText());
        }
        if (showHeadDebug) {
            if (text.length() > 0) {
                text.append("  ||  ");
            }
            text.append(DroneDebug.screenText());
        }
        DroneController.notify(mc, text.toString());
    }

    private static void notify(Minecraft mc, String msg) {
        if (mc.player != null) {
            mc.gui.chatListener().handleOverlay((Component)Component.literal((String)("[FPV] " + msg)));
        }
    }

    public static boolean poll() {
        if (handle == 0L) {
            return false;
        }
        if (showAxisValues) {
            DroneController.recordDebugValues();
        }
        rawRoll = DroneController.read(Axis.ROLL);
        rawPitch = DroneController.read(Axis.PITCH);
        rawYaw = DroneController.read(Axis.YAW);
        rawThrottle = DroneController.read(Axis.THROTTLE);
        roll = DroneController.shape(rawRoll);
        pitch = DroneController.shape(rawPitch);
        yaw = DroneController.shape(rawYaw);
        rollDegPerSec = DroneController.rateDegPerSec(Axis.ROLL, roll);
        pitchDegPerSec = DroneController.rateDegPerSec(Axis.PITCH, pitch);
        yawDegPerSec = DroneController.rateDegPerSec(Axis.YAW, yaw);
        throttle = mode3d ? Math.max(-1.0f, Math.min(1.0f, rawThrottle)) : Math.max(0.0f, Math.min(1.0f, (rawThrottle + 1.0f) * 0.5f));
        return true;
    }

    private static float read(Axis a) {
        int i = a.ordinal();
        return isGamepad ? DroneController.gamepadAxis(gpAxis[i], gpInvert[i]) : DroneController.rawAxis(rawAxis[i], rawInvert[i]);
    }

    private static float gamepadAxis(int axis, boolean invert) {
        float v = DroneController.norm(SDLGamepad.SDL_GetGamepadAxis((long)handle, (int)axis));
        return invert ? -v : v;
    }

    private static float rawAxis(int index, boolean invert) {
        if (index < 0 || index >= SDLJoystick.SDL_GetNumJoystickAxes((long)handle)) {
            return 0.0f;
        }
        float v = DroneController.norm(SDLJoystick.SDL_GetJoystickAxis((long)handle, (int)index));
        return invert ? -v : v;
    }

    private static short sdlValue(Axis a) {
        int i = a.ordinal();
        if (isGamepad) {
            return SDLGamepad.SDL_GetGamepadAxis((long)handle, (int)gpAxis[i]);
        }
        int index = rawAxis[i];
        if (index < 0 || index >= SDLJoystick.SDL_GetNumJoystickAxes((long)handle)) {
            return 0;
        }
        return SDLJoystick.SDL_GetJoystickAxis((long)handle, (int)index);
    }

    private static void recordDebugValues() {
        for (Axis a : Axis.values()) {
            short v;
            int i = a.ordinal();
            DroneController.debugRaw[i] = v = DroneController.sdlValue(a);
            if (v < debugMin[i]) {
                DroneController.debugMin[i] = v;
            }
            if (v <= debugMax[i]) continue;
            DroneController.debugMax[i] = v;
        }
    }

    private static void resetDebugRange() {
        Arrays.fill(debugRaw, (short)0);
        Arrays.fill(debugMin, (short)Short.MAX_VALUE);
        Arrays.fill(debugMax, (short)Short.MIN_VALUE);
    }

    private static String debugText() {
        StringBuilder sb = new StringBuilder();
        String[] labels = new String[]{"R", "P", "Y", "T"};
        for (Axis a : Axis.values()) {
            int i = a.ordinal();
            if (i > 0) {
                sb.append("  ");
            }
            sb.append(String.format("%s %d [%d..%d]", labels[i], debugRaw[i], debugMin[i], debugMax[i]));
        }
        return sb.toString();
    }

    private static float norm(short raw) {
        return Math.max(-1.0f, Math.min(1.0f, (float)raw / 32767.0f));
    }

    private static float shape(float v) {
        float a = Math.abs(v);
        if (a < deadzone) {
            return 0.0f;
        }
        a = (a - deadzone) / (1.0f - deadzone);
        return Math.copySign(Math.min(a, 1.0f), v);
    }

    public static float rateDegPerSec(Axis axis, float stick) {
        int i = axis.ordinal();
        float abs = Math.abs(stick);
        float shaped = stick * abs * abs * expo[i] + stick * (1.0f - expo[i]);
        float rate = rcRate[i];
        if (rate > 2.0f) {
            rate += 14.54f * (rate - 2.0f);
        }
        float degPerSec = 200.0f * rate * shaped;
        if (superRate[i] > 0.0f) {
            float factor = 1.0f / Math.max(0.01f, Math.min(1.0f, 1.0f - abs * superRate[i]));
            degPerSec *= factor;
        }
        return degPerSec;
    }

    public static float getRcRate(Axis a) {
        DroneController.ensureConfig();
        return rcRate[a.ordinal()];
    }

    public static float getSuperRate(Axis a) {
        DroneController.ensureConfig();
        return superRate[a.ordinal()];
    }

    public static float getExpo(Axis a) {
        DroneController.ensureConfig();
        return expo[a.ordinal()];
    }

    public static void setRcRate(Axis a, float v) {
        DroneController.ensureConfig();
        DroneController.rcRate[a.ordinal()] = DroneController.clamp(v, 0.01f, 2.55f);
    }

    public static void setSuperRate(Axis a, float v) {
        DroneController.ensureConfig();
        DroneController.superRate[a.ordinal()] = DroneController.clamp(v, 0.0f, 0.99f);
    }

    public static void setExpo(Axis a, float v) {
        DroneController.ensureConfig();
        DroneController.expo[a.ordinal()] = DroneController.clamp(v, 0.0f, 1.0f);
    }

    public static void resetRates() {
        Arrays.fill(rcRate, 0.67f);
        Arrays.fill(superRate, 0.2f);
        Arrays.fill(expo, 0.67f);
        if (configLoaded) {
            DroneController.saveConfig();
        }
    }

    private static float clamp(float v, float min, float max) {
        return Math.max(min, Math.min(max, v));
    }

    public static Component bindingName(Axis a) {
        DroneController.ensureConfig();
        int i = a.ordinal();
        if (isGamepad) {
            int ax = gpAxis[i];
            if (ax >= 0 && ax < 6) {
                return Component.translatable((String)GP_AXIS_NAME_KEYS[ax]);
            }
            return Component.translatable((String)"fpvdrone.axis.raw", (Object[])new Object[]{ax});
        }
        return Component.translatable((String)"fpvdrone.axis.raw", (Object[])new Object[]{rawAxis[i]});
    }

    public static boolean isInverted(Axis a) {
        DroneController.ensureConfig();
        return isGamepad ? gpInvert[a.ordinal()] : rawInvert[a.ordinal()];
    }

    public static void toggleInvert(Axis a) {
        int i = a.ordinal();
        if (isGamepad) {
            DroneController.gpInvert[i] = !gpInvert[i];
        } else {
            DroneController.rawInvert[i] = !rawInvert[i];
        }
        DroneController.saveConfig();
    }

    public static void resetBindings() {
        DroneController.cancelCapture();
        if (isGamepad) {
            System.arraycopy(GP_AXIS_DEFAULT, 0, gpAxis, 0, gpAxis.length);
            System.arraycopy(GP_INVERT_DEFAULT, 0, gpInvert, 0, gpInvert.length);
        } else {
            System.arraycopy(RAW_AXIS_DEFAULT, 0, rawAxis, 0, rawAxis.length);
            System.arraycopy(RAW_INVERT_DEFAULT, 0, rawInvert, 0, rawInvert.length);
        }
        DroneController.saveConfig();
    }

    public static boolean beginCapture(Axis target, Runnable onDone) {
        if (handle == 0L) {
            return false;
        }
        captureTarget = target;
        captureDone = onDone;
        captureBase = DroneController.readAllAxes();
        return true;
    }

    public static void cancelCapture() {
        captureTarget = null;
        captureDone = null;
        captureBase = null;
    }

    public static Axis capturingAxis() {
        return captureTarget;
    }

    private static float[] readAllAxes() {
        int n = isGamepad ? 6 : SDLJoystick.SDL_GetNumJoystickAxes((long)handle);
        float[] v = new float[n];
        for (int i = 0; i < n; ++i) {
            v[i] = isGamepad ? DroneController.norm(SDLGamepad.SDL_GetGamepadAxis((long)handle, (int)i)) : DroneController.norm(SDLJoystick.SDL_GetJoystickAxis((long)handle, (int)i));
        }
        return v;
    }

    private static void updateCapture() {
        int i;
        float[] now = DroneController.readAllAxes();
        int best = -1;
        float bestDelta = 0.0f;
        int n = Math.min(now.length, captureBase.length);
        for (i = 0; i < n; ++i) {
            float d = now[i] - captureBase[i];
            if (!(Math.abs(d) > Math.abs(bestDelta))) continue;
            best = i;
            bestDelta = d;
        }
        if (best < 0 || Math.abs(bestDelta) < 0.5f) {
            return;
        }
        i = captureTarget.ordinal();
        if (isGamepad) {
            DroneController.gpAxis[i] = best;
            DroneController.gpInvert[i] = bestDelta < 0.0f;
        } else {
            DroneController.rawAxis[i] = best;
            DroneController.rawInvert[i] = bestDelta < 0.0f;
        }
        Runnable done = captureDone;
        DroneController.cancelCapture();
        DroneController.saveConfig();
        if (done != null) {
            done.run();
        }
    }

    public static boolean isShowAxisValues() {
        DroneController.ensureConfig();
        return showAxisValues;
    }

    public static void setShowAxisValues(boolean value) {
        DroneController.ensureConfig();
        showAxisValues = value;
        DroneController.resetDebugRange();
        DroneController.saveConfig();
    }

    public static boolean isShowHeadDebug() {
        DroneController.ensureConfig();
        return showHeadDebug;
    }

    public static void setShowHeadDebug(boolean value) {
        DroneController.ensureConfig();
        showHeadDebug = value;
        DroneDebug.setEnabled(value);
        DroneController.saveConfig();
    }

    public static boolean isMode3d() {
        DroneController.ensureConfig();
        return mode3d;
    }

    public static void setMode3d(boolean value) {
        DroneController.ensureConfig();
        mode3d = value;
        DroneController.saveConfig();
    }

    private static Path configPath() {
        return FMLPaths.CONFIGDIR.get().resolve("fpvdrone-controller.properties");
    }

    private static void loadConfig() {
        int i;
        configLoaded = true;
        Path path = DroneController.configPath();
        Properties p = new Properties();
        if (Files.exists(path, new LinkOption[0])) {
            try (InputStream in = Files.newInputStream(path, new OpenOption[0]);){
                p.load(in);
            }
            catch (IOException e) {
                FpvDroneMod.LOGGER.warn("[fpvdrone] could not read controller config", (Throwable)e);
            }
        }
        deadzone = DroneController.getF(p, "deadzone", deadzone);
        for (Axis a : Axis.values()) {
            i = a.ordinal();
            DroneController.rcRate[i] = DroneController.clamp(DroneController.getF(p, a.key + "_rc_rate", rcRate[i]), 0.01f, 2.55f);
            DroneController.superRate[i] = DroneController.clamp(DroneController.getF(p, a.key + "_super_rate", superRate[i]), 0.0f, 0.99f);
            DroneController.expo[i] = DroneController.clamp(DroneController.getF(p, a.key + "_expo", expo[i]), 0.0f, 1.0f);
        }
        selectedId = p.getProperty("selected_controller", "");
        showAxisValues = DroneController.getB(p, "show_axis_values", showAxisValues);
        showHeadDebug = DroneController.getB(p, "show_head_debug", showHeadDebug);
        DroneDebug.setEnabled(showHeadDebug);
        mode3d = DroneController.getB(p, "mode_3d", mode3d);
        for (Axis a : Axis.values()) {
            i = a.ordinal();
            DroneController.gpAxis[i] = DroneController.getI(p, "gp_axis_" + a.key, gpAxis[i]);
            DroneController.gpInvert[i] = DroneController.getB(p, "gp_invert_" + a.key, gpInvert[i]);
            DroneController.rawAxis[i] = DroneController.getI(p, "raw_axis_" + a.key, rawAxis[i]);
            DroneController.rawInvert[i] = DroneController.getB(p, "raw_invert_" + a.key, rawInvert[i]);
        }
        if (!Files.exists(path, new LinkOption[0])) {
            DroneController.saveConfig();
        }
    }

    public static void saveConfig() {
        int i;
        Properties p = new Properties();
        p.setProperty("deadzone", String.valueOf(deadzone));
        for (Axis a : Axis.values()) {
            i = a.ordinal();
            p.setProperty(a.key + "_rc_rate", String.valueOf(rcRate[i]));
            p.setProperty(a.key + "_super_rate", String.valueOf(superRate[i]));
            p.setProperty(a.key + "_expo", String.valueOf(expo[i]));
        }
        p.setProperty("selected_controller", selectedId);
        p.setProperty("show_axis_values", String.valueOf(showAxisValues));
        p.setProperty("show_head_debug", String.valueOf(showHeadDebug));
        p.setProperty("mode_3d", String.valueOf(mode3d));
        for (Axis a : Axis.values()) {
            i = a.ordinal();
            p.setProperty("gp_axis_" + a.key, String.valueOf(gpAxis[i]));
            p.setProperty("gp_invert_" + a.key, String.valueOf(gpInvert[i]));
            p.setProperty("raw_axis_" + a.key, String.valueOf(rawAxis[i]));
            p.setProperty("raw_invert_" + a.key, String.valueOf(rawInvert[i]));
        }
        try (OutputStream out = Files.newOutputStream(DroneController.configPath(), new OpenOption[0]);){
            p.store(out, "FPV Drone controller config. selected_controller empty = auto-detect. gp_* applies to gamepads (SDL axis ids), raw_* to RC/joystick devices (axis index).");
        }
        catch (IOException e) {
            FpvDroneMod.LOGGER.warn("[fpvdrone] could not write controller config", (Throwable)e);
        }
    }

    private static float getF(Properties p, String k, float d) {
        try {
            return Float.parseFloat(p.getProperty(k, String.valueOf(d)).trim());
        }
        catch (NumberFormatException e) {
            return d;
        }
    }

    private static int getI(Properties p, String k, int d) {
        try {
            return Integer.parseInt(p.getProperty(k, String.valueOf(d)).trim());
        }
        catch (NumberFormatException e) {
            return d;
        }
    }

    private static boolean getB(Properties p, String k, boolean d) {
        return Boolean.parseBoolean(p.getProperty(k, String.valueOf(d)).trim());
    }

    static {
        showAxisValues = false;
        showHeadDebug = false;
        mode3d = false;
        debugRaw = new short[4];
        debugMin = new short[4];
        debugMax = new short[4];
        handle = 0L;
        name = "";
        connectedId = "";
        deadzone = 0.05f;
        rcRate = new float[Axis.values().length];
        superRate = new float[Axis.values().length];
        expo = new float[Axis.values().length];
        DroneController.resetRates();
        selectedId = "";
        GP_AXIS_DEFAULT = new int[]{2, 3, 0, 1};
        GP_INVERT_DEFAULT = new boolean[]{false, true, false, true};
        RAW_AXIS_DEFAULT = new int[]{0, 1, 3, 2};
        RAW_INVERT_DEFAULT = new boolean[]{false, true, false, false};
        gpAxis = (int[])GP_AXIS_DEFAULT.clone();
        gpInvert = (boolean[])GP_INVERT_DEFAULT.clone();
        rawAxis = (int[])RAW_AXIS_DEFAULT.clone();
        rawInvert = (boolean[])RAW_INVERT_DEFAULT.clone();
        GP_AXIS_NAME_KEYS = new String[6];
        for (int i = 0; i < 6; ++i) {
            DroneController.GP_AXIS_NAME_KEYS[i] = "fpvdrone.axis.gamepad." + i;
        }
        configLoaded = false;
    }

    public record Device(String id, String name, boolean gamepad) {
    }

    public static enum Axis {
        ROLL("roll"),
        PITCH("pitch"),
        YAW("yaw"),
        THROTTLE("throttle");

        public final String key;

        private Axis(String key) {
            this.key = key;
        }
    }
}
