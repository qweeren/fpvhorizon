/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race.db;

import java.sql.Connection;

@FunctionalInterface
public static interface Database.SqlFunction<T> {
    public T apply(Connection var1) throws Exception;
}
