/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  org.joml.Quaternionf
 *  org.joml.Vector3f
 *  org.joml.Vector3fc
 */
package com.taylorgoolsby.fpvdrone;

import com.taylorgoolsby.fpvdrone.DroneBuild;
import com.taylorgoolsby.fpvdrone.PhysicsSettings;
import org.joml.Quaternionf;
import org.joml.Vector3f;
import org.joml.Vector3fc;

public class AdvancedPhysicsCore {
    public static final float GRAVITY = 9.8f;
    private static final float INCHES_PER_METER = 39.3701f;
    private static final float REF_TEMP = 293.15f;
    private static final float VAC = 1.256637E-6f;
    private static final float MAX_TIME_STEP = 0.0078125f;
    private static final float MIN_TIME_STEP = 0.001f;
    private static final long MAX_STEP_BUDGET_NANOS = 66000000L;
    private static final int PROP_SECTIONS = 5;
    private static final float FALLBACK_RESISTANCE = 0.03f;
    private int appliedBuildVersion = -1;
    private int appliedPhysicsVersion = -1;
    private float airDensity;
    private float droneDensity;
    private float airDragCoefficient;
    private float resistancePer1000Ft;
    private float tempCoeff;
    private float specificHeat;
    private float heatTransfer;
    private float droneMass;
    private int batteryCells;
    private int blades;
    private float propRadius;
    private float propPitch;
    private float propWidth;
    private float motorWidth;
    private float motorHeight;
    private float motorMass;
    private float bellMass;
    private float bladeMass;
    private float kv;
    private float refResistance;
    private float heatCap;
    private float motorSurfaceArea;
    private float inertia;
    private float dragFactor;
    private float maxTimeStep;
    private float hoverThrottle;
    private final float[] internalEnergy = new float[4];
    private final float[] temp = new float[4];
    private final float[] motorVel = new float[4];
    private final float[] motorPos = new float[4];
    private final Vector3f velocity = new Vector3f();
    private final Vector3f droneUp = new Vector3f(0.0f, 1.0f, 0.0f);
    private final Vector3f droneLeft = new Vector3f(1.0f, 0.0f, 0.0f);

    public AdvancedPhysicsCore() {
        this.applyBuildIfChanged();
        this.resetState(0.0f);
    }

    private void applyBuildIfChanged() {
        int buildVersion = DroneBuild.getVersion();
        int physicsVersion = PhysicsSettings.getVersion();
        if (buildVersion == this.appliedBuildVersion && physicsVersion == this.appliedPhysicsVersion) {
            return;
        }
        this.appliedBuildVersion = buildVersion;
        this.appliedPhysicsVersion = physicsVersion;
        this.airDensity = PhysicsSettings.get(PhysicsSettings.Param.AIR_DENSITY);
        this.droneDensity = PhysicsSettings.get(PhysicsSettings.Param.DRONE_DENSITY);
        this.airDragCoefficient = PhysicsSettings.get(PhysicsSettings.Param.DRAG_COEFFICIENT);
        this.resistancePer1000Ft = PhysicsSettings.get(PhysicsSettings.Param.RESISTANCE_PER_1000_FT);
        this.tempCoeff = PhysicsSettings.get(PhysicsSettings.Param.TEMP_COEFF);
        this.specificHeat = PhysicsSettings.get(PhysicsSettings.Param.SPECIFIC_HEAT);
        this.heatTransfer = PhysicsSettings.get(PhysicsSettings.Param.HEAT_TRANSFER);
        this.droneMass = DroneBuild.get(DroneBuild.Param.DRONE_MASS);
        this.batteryCells = Math.round(DroneBuild.get(DroneBuild.Param.BATTERY_CELLS));
        this.blades = Math.round(DroneBuild.get(DroneBuild.Param.BLADES));
        this.propRadius = DroneBuild.get(DroneBuild.Param.PROP_RADIUS);
        this.propPitch = DroneBuild.get(DroneBuild.Param.PROP_PITCH);
        this.propWidth = DroneBuild.get(DroneBuild.Param.PROP_WIDTH);
        this.motorWidth = DroneBuild.get(DroneBuild.Param.MOTOR_WIDTH);
        this.motorHeight = DroneBuild.get(DroneBuild.Param.MOTOR_HEIGHT);
        this.motorMass = DroneBuild.motorMass();
        this.bellMass = DroneBuild.bellMass();
        this.bladeMass = DroneBuild.bladeMass();
        this.kv = DroneBuild.get(DroneBuild.Param.MOTOR_KV) * 0.10471976f;
        float R = this.motorWidth * 0.5f;
        float poles = AdvancedPhysicsCore.getPoles(this.motorWidth);
        float slots = AdvancedPhysicsCore.getSlots(this.motorWidth);
        this.heatCap = this.specificHeat * this.motorMass;
        this.motorSurfaceArea = (float)Math.PI * 2 * R * this.motorHeight + (float)Math.PI * 2 * R * R;
        float resistance = this.computeRefResistance(R, this.motorHeight, this.kv, poles, slots);
        if (Float.isNaN(resistance) || Float.isInfinite(resistance) || resistance < 0.001f) {
            resistance = 0.03f;
        }
        this.refResistance = resistance;
        this.inertia = this.computeMotorInertia();
        float radius = (float)Math.cbrt(3.0f * this.droneMass / (this.droneDensity * 4.0f * (float)Math.PI));
        float area = (float)Math.PI * radius * radius;
        this.dragFactor = this.airDensity * area * this.airDragCoefficient / 2.0f;
        float timeConstant = this.refResistance * this.kv * this.kv * this.inertia;
        this.maxTimeStep = Math.max(0.001f, Math.min(0.0078125f, timeConstant));
        this.hoverThrottle = this.findHoverThrottle();
    }

    public void reset(float throttle) {
        this.applyBuildIfChanged();
        this.resetState(throttle);
    }

    public float getHoverThrottle() {
        this.applyBuildIfChanged();
        return this.hoverThrottle;
    }

    private void resetState(float throttle) {
        for (int i = 0; i < 4; ++i) {
            this.internalEnergy[i] = 0.0f;
            this.temp[i] = 293.15f;
            this.motorVel[i] = 0.0f;
            this.motorPos[i] = 0.0f;
        }
        this.velocity.zero();
        this.droneUp.set(0.0f, 1.0f, 0.0f);
        this.droneLeft.set(1.0f, 0.0f, 0.0f);
        float vbat = this.getBatteryVoltage(AdvancedPhysicsCore.clampThrottle(throttle));
        for (int motor = 0; motor < 4; ++motor) {
            float useVbat = (motor % 2 == 0 ? -1.0f : 1.0f) * vbat;
            this.motorVel[motor] = this.steadyMotorVel(motor, useVbat, this.temp[motor]);
        }
    }

    private float steadyMotorVel(int motorNumber, float vbat, float temp) {
        if (vbat == 0.0f) {
            return 0.0f;
        }
        float direction = Math.signum(vbat);
        float lo = 0.0f;
        float hi = direction * this.kv * (float)this.batteryCells * 4.2f;
        for (int i = 0; i < 40; ++i) {
            float mid = (lo + hi) * 0.5f;
            float accel = this.getRotAccel(motorNumber, mid, vbat, temp);
            if (accel * direction > 0.0f) {
                lo = mid;
                continue;
            }
            hi = mid;
        }
        return (lo + hi) * 0.5f;
    }

    public void step(float dt, float throttle, Quaternionf orientation, Vector3f vel) {
        long tic = System.nanoTime();
        this.applyBuildIfChanged();
        orientation.transform((Vector3fc)new Vector3f(0.0f, 1.0f, 0.0f), this.droneUp);
        Vector3f droneLook = orientation.transform((Vector3fc)new Vector3f(0.0f, 0.0f, -1.0f), new Vector3f());
        this.droneUp.cross((Vector3fc)droneLook, this.droneLeft);
        this.velocity.set((Vector3fc)vel);
        float vbat = this.getBatteryVoltage(AdvancedPhysicsCore.clampThrottle(throttle));
        Vector3f gravityForce = new Vector3f(0.0f, -9.8f * this.droneMass, 0.0f);
        int maxIterations = (int)(dt / this.maxTimeStep) + 10;
        float timeProcessed = 0.0f;
        for (int i = 0; i < maxIterations && dt - timeProcessed > 1.0E-5f; ++i) {
            float timeStep = Math.min(this.maxTimeStep, dt - timeProcessed);
            float speed = this.velocity.length();
            Vector3f netForce = AdvancedPhysicsCore.normalizedOrZero(this.velocity).mul(-speed * speed * this.dragFactor).add((Vector3fc)gravityForce);
            for (int motor = 0; motor < 4; ++motor) {
                Vector3f[] results = this.getTotalForcesOnProp(motor, this.motorVel[motor]);
                netForce.add((Vector3fc)results[0]);
                float useVbat = (motor % 2 == 0 ? -1.0f : 1.0f) * vbat;
                float nextMotorVel = this.advanceMotorVel(motor, this.motorVel[motor], timeStep, useVbat, this.temp[motor]);
                this.internalEnergy[motor] = this.getNextInternalEnergy(motor, this.internalEnergy[motor], this.temp[motor], useVbat, this.motorVel[motor], timeStep);
                this.temp[motor] = this.getNextTemp(this.internalEnergy[motor]);
                this.motorPos[motor] = AdvancedPhysicsCore.wrapAngle(this.motorPos[motor] + this.motorVel[motor] * timeStep);
                this.motorVel[motor] = nextMotorVel;
            }
            this.velocity.fma(timeStep / this.droneMass, (Vector3fc)netForce);
            timeProcessed += timeStep;
            if (System.nanoTime() - tic > 66000000L) break;
        }
        vel.set((Vector3fc)this.velocity);
    }

    public float[] getMotorVel() {
        return this.motorVel;
    }

    public float[] getMotorPos() {
        return this.motorPos;
    }

    public float[] getMotorTemp() {
        return this.temp;
    }

    public boolean isOverheat() {
        float averageRes = 0.0f;
        for (int i = 0; i < 4; ++i) {
            averageRes += this.getHotResistance(this.temp[i]);
        }
        return this.refResistance / (averageRes /= 4.0f) < 0.5f;
    }

    private float findHoverThrottle() {
        float weight = this.droneMass * 9.8f;
        float lo = 0.0f;
        float hi = 1.0f;
        for (int i = 0; i < 16; ++i) {
            float mid = (lo + hi) * 0.5f;
            this.resetState(mid);
            if (this.totalThrust() < weight) {
                lo = mid;
                continue;
            }
            hi = mid;
        }
        this.resetState(1.0f);
        if (this.totalThrust() < weight) {
            return 1.0f;
        }
        return (lo + hi) * 0.5f;
    }

    private float totalThrust() {
        float thrust = 0.0f;
        for (int motor = 0; motor < 4; ++motor) {
            Vector3f force = this.getTotalForcesOnProp(motor, this.motorVel[motor])[0];
            thrust += force.dot((Vector3fc)this.droneUp);
        }
        return thrust;
    }

    private static Vector3f getMotorToProp(float rotationAngle, Vector3f worldDroneUp, Vector3f worldDroneRight) {
        return new Quaternionf().rotationAxis(rotationAngle, worldDroneUp.x, worldDroneUp.y, worldDroneUp.z).transform((Vector3fc)worldDroneRight, new Vector3f());
    }

    private static Vector3f getWorldPropVelocity(Vector3f worldDroneUp, Vector3f worldDroneVelocity, Vector3f motorToProp, float angularSpeed, float sectionRadius) {
        Vector3f propVelocity = AdvancedPhysicsCore.normalizedOrZero(worldDroneUp.cross((Vector3fc)motorToProp, new Vector3f()));
        return propVelocity.mul(angularSpeed * sectionRadius).add((Vector3fc)worldDroneVelocity);
    }

    private static Vector3f getFreeVelRelPropPlanar(Vector3f worldPropVelocity, Vector3f worldVelocityFree, Vector3f motorToProp) {
        Vector3f freeVelRelProp = worldVelocityFree.sub((Vector3fc)worldPropVelocity, new Vector3f());
        Vector3f normalProj = AdvancedPhysicsCore.project(freeVelRelProp, motorToProp);
        return freeVelRelProp.sub((Vector3fc)normalProj);
    }

    private static float getAoA(int motorNumber, float propPitch, float sectionRadius, Vector3f freeVelRelPropPlanar, Vector3f worldDroneUp, Vector3f motorToProp) {
        float sign;
        float pitchAngle = (float)Math.atan2(propPitch, (float)Math.PI * 2 * sectionRadius);
        Vector3f windDirection = AdvancedPhysicsCore.normalizedOrZero(freeVelRelPropPlanar.mul(-1.0f, new Vector3f()));
        Vector3f forwardSpinDirection = worldDroneUp.cross((Vector3fc)motorToProp, new Vector3f());
        if (motorNumber % 2 == 0) {
            forwardSpinDirection.mul(-1.0f);
        }
        if ((sign = Math.signum(worldDroneUp.dot((Vector3fc)windDirection))) == 0.0f) {
            sign = 1.0f;
        }
        float windAngle = sign * AdvancedPhysicsCore.angleBetween(forwardSpinDirection, windDirection);
        float angleOfAttack = pitchAngle - windAngle;
        if (motorNumber % 2 == 0) {
            angleOfAttack *= -1.0f;
        }
        return angleOfAttack;
    }

    private static float getPropDragCoefficient(float aoa) {
        float min = 0.4f;
        float max = 1.8f;
        float minWrapped = (float)Math.pow(min, 1.15);
        float maxWrapped = (float)Math.pow(max, 1.15);
        float generalShape = -((float)Math.cos(aoa * 2.0f)) + 1.0f;
        float scaledAndOffset = generalShape / 2.0f * (maxWrapped - minWrapped) + minWrapped;
        return (float)Math.pow(scaledAndOffset, 0.8695652173913044);
    }

    private static float getPropLiftCoefficient(float aoa) {
        aoa = aoa < 0.0f ? (float)Math.PI + aoa % (float)Math.PI : (aoa %= (float)Math.PI);
        float pi = (float)Math.PI;
        float alp = 7.5f * (pi / 180.0f);
        float sqrt2OverPi = (float)Math.sqrt(2.0f / pi);
        float naca = sqrt2OverPi * aoa * aoa * (float)Math.exp(-(aoa * aoa) / (2.0f * alp * alp)) / (alp * alp * alp);
        float antiaoa = pi - aoa;
        float antinaca = -sqrt2OverPi * antiaoa * antiaoa * (float)Math.exp(-(antiaoa * antiaoa) / (2.0f * alp * alp)) / (alp * alp * alp);
        float plate = (float)Math.sin(aoa * 2.0f);
        return plate + (naca /= 7.0f) + (antinaca /= 7.0f);
    }

    private Vector3f getPropDragForce(Vector3f freeVelRelPropPlanar, float dragCoefficient, float aoa, float sectionLength, float sectionWidth) {
        float flowSpeed = freeVelRelPropPlanar.length();
        float thickness = 0.002f;
        float referenceArea = Math.abs(sectionLength * sectionWidth * (float)Math.sin(aoa)) + Math.abs(sectionLength * thickness * (float)Math.cos(aoa));
        float forceMag = dragCoefficient * this.airDensity * flowSpeed * flowSpeed * referenceArea / 2.0f;
        return AdvancedPhysicsCore.normalizedOrZero(freeVelRelPropPlanar).mul(forceMag);
    }

    private Vector3f getPropLiftForce(Vector3f freeVelRelPropPlanar, Vector3f motorToProp, float liftCoefficient, float sectionLength, float sectionWidth) {
        float flowSpeed = freeVelRelPropPlanar.length();
        float referenceArea = sectionLength * sectionWidth;
        float forceMag = liftCoefficient * this.airDensity * flowSpeed * flowSpeed * referenceArea / 2.0f;
        Vector3f windDirection = AdvancedPhysicsCore.normalizedOrZero(freeVelRelPropPlanar.mul(-1.0f, new Vector3f()));
        return AdvancedPhysicsCore.normalizedOrZero(motorToProp).cross((Vector3fc)windDirection).mul(forceMag);
    }

    private Vector3f[] getTotalForcesOnProp(int motorNumber, float rotvel) {
        Vector3f netForce = new Vector3f();
        Vector3f netDragTorque = new Vector3f();
        Vector3f worldVelocityFree = new Vector3f();
        Vector3f droneRight = this.droneLeft.mul(-1.0f, new Vector3f());
        float initRotationAngle = this.motorPos[motorNumber];
        float sectionLength = this.propRadius / 5.0f;
        for (int i = 0; i < this.blades; ++i) {
            float rotationAngle = initRotationAngle + (float)Math.PI * 2 * (float)i / (float)this.blades;
            Vector3f motorToProp = AdvancedPhysicsCore.getMotorToProp(rotationAngle, this.droneUp, droneRight);
            for (int j = 0; j < 5; ++j) {
                float sectionRadius = ((float)j + 0.5f) / 5.0f * this.propRadius;
                if (sectionRadius < this.motorWidth * 0.5f) continue;
                float sectionWidth = this.propWidth;
                if (sectionRadius < this.motorWidth) {
                    sectionWidth *= sectionRadius / this.motorWidth;
                }
                Vector3f worldPropVelocity = AdvancedPhysicsCore.getWorldPropVelocity(this.droneUp, this.velocity, motorToProp, rotvel, sectionRadius);
                Vector3f freeVelRelPropPlanar = AdvancedPhysicsCore.getFreeVelRelPropPlanar(worldPropVelocity, worldVelocityFree, motorToProp);
                float aoa = AdvancedPhysicsCore.getAoA(motorNumber, this.propPitch, sectionRadius, freeVelRelPropPlanar, this.droneUp, motorToProp);
                float dragCoefficient = AdvancedPhysicsCore.getPropDragCoefficient(aoa);
                float liftCoefficient = AdvancedPhysicsCore.getPropLiftCoefficient(aoa);
                Vector3f dragForce = this.getPropDragForce(freeVelRelPropPlanar, dragCoefficient, aoa, sectionLength, sectionWidth);
                Vector3f liftForce = this.getPropLiftForce(freeVelRelPropPlanar, motorToProp, liftCoefficient, sectionLength, sectionWidth);
                netDragTorque.add((Vector3fc)motorToProp.cross((Vector3fc)dragForce, new Vector3f()).mul(sectionRadius));
                netForce.add((Vector3fc)dragForce);
                netForce.add((Vector3fc)liftForce);
            }
        }
        return new Vector3f[]{netForce, netDragTorque};
    }

    private float dflux_dt(float x, float slotW, float R, float h, float m, float rotvel) {
        float gap = 0.00159f;
        float xlow = -slotW / 2.0f;
        float xhigh = slotW / 2.0f;
        float gmin = gap * gap + (x + xlow) * (x + xlow);
        float gmax = gap * gap + (x + xhigh) * (x + xhigh);
        float c1 = 1.256637E-6f * h * R * rotvel * m / ((float)Math.PI * 4);
        float c2 = 3.0f * gap * gap;
        return (float)((double)c1 * ((double)c2 * (Math.pow(gmax, -2.5) - Math.pow(gmin, -2.5)) - (Math.pow(gmax, -1.5) - Math.pow(gmin, -1.5))));
    }

    private static int getPoles(float motorWidth) {
        return motorWidth >= 0.018f ? 14 : 12;
    }

    private static int getSlots(float motorWidth) {
        return motorWidth >= 0.018f ? 12 : 9;
    }

    private static float getSlotW(float R, float slots) {
        return (float)Math.PI * 2 * R / slots - 5.0E-4f;
    }

    private static float getMagW(float R, float poles) {
        return (float)Math.PI * 2 * R / poles - 5.0E-4f;
    }

    private float getPoleMagneticMoment(float R, float h, float poles) {
        float magW = AdvancedPhysicsCore.getMagW(R, poles);
        float magD = 0.001f;
        return 1177746.6f * magW * h * magD;
    }

    private float getN(float R, float h, float kv, float poles, float slots) {
        float slotW = AdvancedPhysicsCore.getSlotW(R, slots);
        float magW = AdvancedPhysicsCore.getMagW(R, poles);
        float m = this.getPoleMagneticMoment(R, h, poles);
        float x1 = -magW + magW / 2.0f;
        float x2 = magW / 2.0f;
        float x3 = magW + magW / 2.0f;
        float d1 = this.dflux_dt(x1, slotW, R, h, m, 1.0f);
        float d2 = this.dflux_dt(x2, slotW, R, h, -m, 1.0f);
        float d3 = this.dflux_dt(x3, slotW, R, h, m, 1.0f);
        float d = d1 + d2 + d3;
        return Math.round(1.0f / (d * kv));
    }

    private float computeRefResistance(float R, float h, float kv, float poles, float slots) {
        float slotW = AdvancedPhysicsCore.getSlotW(R, slots);
        float N = this.getN(R, h, kv, poles, slots);
        float loopPeri = 2.0f * h + 2.0f * slotW;
        float wireLengthInches = N * loopPeri * 39.3701f;
        return wireLengthInches / 12.0f / 1000.0f * this.resistancePer1000Ft;
    }

    private float computeMotorInertia() {
        float bladeLength = this.propRadius;
        float propInertia = (float)this.blades / 12.0f * this.bladeMass * (4.0f * bladeLength * bladeLength + this.propWidth * this.propWidth);
        float bellInertia = this.bellMass * (this.motorWidth * 0.5f) * (this.motorWidth * 0.5f);
        return propInertia + bellInertia;
    }

    private float getHotResistance(float temp) {
        return this.refResistance * (1.0f + this.tempCoeff * (temp - 293.15f));
    }

    private float getCurrent(float resistance, float vbat, float rotvel) {
        float backEMF = -rotvel / this.kv;
        return (vbat + backEMF) / resistance;
    }

    private float getTorque(float current) {
        return current / this.kv;
    }

    private float getRotAccel(int motorNumber, float rotvel, float vbat, float temp) {
        float resistance = this.getHotResistance(temp);
        float current = this.getCurrent(resistance, vbat, rotvel);
        float torque = this.getTorque(current);
        float dragTorque = this.getTotalForcesOnProp(motorNumber, rotvel)[1].dot((Vector3fc)this.droneUp);
        return (torque + dragTorque) / this.inertia;
    }

    private float advanceMotorVel(int motorNumber, float rotvel, float timeStep, float vbat, float temp) {
        float k4;
        float k3;
        float k2;
        float k1 = this.getRotAccel(motorNumber, rotvel, vbat, temp);
        float nextVel = rotvel + timeStep * (k1 + 2.0f * (k2 = this.getRotAccel(motorNumber, rotvel + timeStep / 2.0f * k1, vbat, temp)) + 2.0f * (k3 = this.getRotAccel(motorNumber, rotvel + timeStep / 2.0f * k2, vbat, temp)) + (k4 = this.getRotAccel(motorNumber, rotvel + timeStep * k3, vbat, temp))) / 6.0f;
        if (Float.isInfinite(nextVel) || Float.isNaN(nextVel)) {
            nextVel = 0.0f;
        }
        float maxSpeed = this.kv * (float)this.batteryCells * 4.2f;
        if (Math.abs(nextVel) > maxSpeed) {
            nextVel = maxSpeed * Math.signum(nextVel);
        }
        return nextVel;
    }

    private float getBatteryVoltage(float throt) {
        float sag = 4.0f + Math.abs(throt) * -0.4000001f;
        return sag * (float)this.batteryCells * throt;
    }

    private float getInternalEnergyLost(float temp, float timeStep) {
        float timeConstant = this.heatCap / (this.heatTransfer * this.motorSurfaceArea);
        float cooledTemp = 293.15f + (temp - 293.15f) * (float)Math.exp(-timeStep / timeConstant);
        return this.heatCap * (cooledTemp - temp);
    }

    private float getInternalEnergyGained(float vbat, float rotvel, float timeStep) {
        float current = this.getCurrent(this.refResistance, vbat, rotvel);
        float heatingPower = current * current * this.refResistance;
        return heatingPower * timeStep;
    }

    private float getNextInternalEnergy(int motorNumber, float internalEnergy, float tempIn, float vbat, float rotvel, float timeStep) {
        float nextEnergy = internalEnergy + this.getInternalEnergyLost(tempIn, timeStep) + this.getInternalEnergyGained(vbat, rotvel, timeStep);
        if (Float.isInfinite(nextEnergy) || Float.isNaN(nextEnergy)) {
            nextEnergy = 0.0f;
            this.temp[motorNumber] = 0.0f;
        }
        return nextEnergy;
    }

    private float getNextTemp(float nextInternalEnergy) {
        return nextInternalEnergy / this.heatCap + 293.15f;
    }

    private static Vector3f normalizedOrZero(Vector3f v) {
        float lengthSquared = v.lengthSquared();
        Vector3f result = new Vector3f((Vector3fc)v);
        if (lengthSquared == 0.0f) {
            return result.zero();
        }
        return result.mul(1.0f / (float)Math.sqrt(lengthSquared));
    }

    private static Vector3f project(Vector3f a, Vector3f b) {
        float bLengthSquared = b.lengthSquared();
        if (bLengthSquared == 0.0f) {
            return new Vector3f();
        }
        return new Vector3f((Vector3fc)b).mul(a.dot((Vector3fc)b) / bLengthSquared);
    }

    private static float angleBetween(Vector3f a, Vector3f b) {
        float dot = Math.max(-1.0f, Math.min(1.0f, a.dot((Vector3fc)b)));
        return (float)Math.acos(dot);
    }

    private static float wrapAngle(float angle) {
        float twoPi = (float)Math.PI * 2;
        float wrapped = angle % twoPi;
        return wrapped < 0.0f ? wrapped + twoPi : wrapped;
    }

    private static float clampThrottle(float v) {
        return Math.max(-1.0f, Math.min(1.0f, v));
    }
}
