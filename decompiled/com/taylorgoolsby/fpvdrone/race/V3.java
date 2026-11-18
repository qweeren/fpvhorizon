/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race;

public final class V3 {
    public static final V3 ZERO = new V3(0.0f, 0.0f, 0.0f);
    public final float x;
    public final float y;
    public final float z;

    public V3(float x, float y, float z) {
        this.x = x;
        this.y = y;
        this.z = z;
    }

    public V3 add(V3 o) {
        return new V3(this.x + o.x, this.y + o.y, this.z + o.z);
    }

    public V3 sub(V3 o) {
        return new V3(this.x - o.x, this.y - o.y, this.z - o.z);
    }

    public V3 mul(float s) {
        return new V3(this.x * s, this.y * s, this.z * s);
    }

    public float dot(V3 o) {
        return this.x * o.x + this.y * o.y + this.z * o.z;
    }

    public V3 cross(V3 o) {
        return new V3(this.y * o.z - this.z * o.y, this.z * o.x - this.x * o.z, this.x * o.y - this.y * o.x);
    }

    public float lengthSquared() {
        return this.x * this.x + this.y * this.y + this.z * this.z;
    }

    public float length() {
        return (float)Math.sqrt(this.lengthSquared());
    }

    public V3 project(V3 other) {
        float n = this.dot(other);
        float d = other.lengthSquared();
        return other.mul(n / d);
    }

    public V3 normalized() {
        float l = this.lengthSquared();
        if (l == 1.0f || l == 0.0f) {
            return this;
        }
        float inv = 1.0f / (float)Math.sqrt(l);
        return new V3(this.x * inv, this.y * inv, this.z * inv);
    }

    public float angleBetween(V3 o) {
        return V3.acosClamped(this.dot(o));
    }

    public static float acosClamped(float v) {
        if (-1.0f < v) {
            if (v < 1.0f) {
                return (float)Math.acos(v);
            }
            return 0.0f;
        }
        return (float)Math.PI;
    }

    public String toString() {
        return "(" + this.x + ", " + this.y + ", " + this.z + ")";
    }
}
