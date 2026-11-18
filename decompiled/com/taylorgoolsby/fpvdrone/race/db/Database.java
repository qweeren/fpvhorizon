/*
 * Decompiled with CFR 0.152.
 */
package com.taylorgoolsby.fpvdrone.race.db;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileAttribute;
import java.sql.Connection;
import java.sql.Driver;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Properties;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Consumer;

public final class Database {
    private static final String DRIVER_CLASS = "org.sqlite.JDBC";
    private static ExecutorService executor;
    private static Connection connection;
    private static String url;
    private static Consumer<String> logInfo;

    private Database() {
    }

    public static void setLoggers(Consumer<String> info) {
        logInfo = info;
    }

    public static boolean isInitialized() {
        return executor != null;
    }

    public static synchronized void init(Path gameDir) throws Exception {
        if (executor != null) {
            return;
        }
        Path dir = gameDir.resolve("fpvdrone");
        Files.createDirectories(dir, new FileAttribute[0]);
        Path file = dir.resolve("races.sqlite");
        url = "jdbc:sqlite:" + String.valueOf(file.toAbsolutePath());
        logInfo.accept("Race database: SQLite at " + String.valueOf(file.toAbsolutePath()));
        executor = Executors.newSingleThreadExecutor(r -> {
            Thread t = new Thread(r, "fpvdrone-db");
            t.setDaemon(true);
            return t;
        });
        try {
            Database.supply(Database::createTables).join();
        }
        catch (CompletionException e) {
            Database.shutdown();
            Throwable cause = e.getCause() != null ? e.getCause() : e;
            throw new Exception(Database.describe(cause), cause);
        }
    }

    private static String describe(Throwable t) {
        return t.getClass().getSimpleName() + ": " + t.getMessage();
    }

    public static synchronized void shutdown() {
        if (executor == null) {
            return;
        }
        ExecutorService ex = executor;
        executor = null;
        ex.execute(() -> {
            try {
                if (connection != null) {
                    connection.close();
                }
            }
            catch (SQLException sQLException) {
                // empty catch block
            }
            connection = null;
        });
        ex.shutdown();
    }

    public static <T> CompletableFuture<T> supply(SqlFunction<T> fn) {
        ExecutorService ex = executor;
        if (ex == null) {
            return CompletableFuture.failedFuture(new IllegalStateException("The race database is not available."));
        }
        CompletableFuture future = new CompletableFuture();
        ex.execute(() -> {
            try {
                future.complete(fn.apply(Database.getConnection()));
            }
            catch (Throwable t) {
                future.completeExceptionally(t);
            }
        });
        return future;
    }

    private static Driver loadDriver() throws Exception {
        ClassLoader[] loaders = new ClassLoader[]{Database.class.getClassLoader(), Thread.currentThread().getContextClassLoader(), ClassLoader.getSystemClassLoader()};
        Throwable first = null;
        for (ClassLoader loader : loaders) {
            if (loader == null) continue;
            try {
                return (Driver)Class.forName(DRIVER_CLASS, true, loader).getDeclaredConstructor(new Class[0]).newInstance(new Object[0]);
            }
            catch (Throwable t) {
                if (first != null) continue;
                first = t;
            }
        }
        String classPath = System.getProperty("java.class.path", "");
        String modulePath = System.getProperty("jdk.module.path", "");
        throw new SQLException("The SQLite driver (org.sqlite.JDBC) could not be loaded. Is sqlite-jdbc on the classpath or bundled in the mod jar? (" + Database.describe(first) + ") [class path has sqlite: " + classPath.toLowerCase().contains("sqlite") + ", module path has sqlite: " + modulePath.toLowerCase().contains("sqlite") + "]", first);
    }

    private static Connection getConnection() throws Exception {
        Connection c;
        if (connection != null) {
            try {
                if (!connection.isClosed() && connection.isValid(2)) {
                    return connection;
                }
            }
            catch (SQLException sQLException) {
                // empty catch block
            }
            try {
                connection.close();
            }
            catch (SQLException sQLException) {
                // empty catch block
            }
            connection = null;
        }
        if ((c = Database.loadDriver().connect(url, new Properties())) == null) {
            throw new SQLException("The SQLite driver rejected URL " + url);
        }
        try (Statement s = c.createStatement();){
            s.execute("PRAGMA journal_mode = WAL");
            s.execute("PRAGMA busy_timeout = 5000");
        }
        connection = c;
        return c;
    }

    private static Void createTables(Connection c) throws SQLException {
        try (Statement s = c.createStatement();){
            s.execute("CREATE TABLE IF NOT EXISTS RaceTrack (raceTrackId VARCHAR(32) NOT NULL, ownerUserId VARCHAR(32) NOT NULL, name VARCHAR(32) NOT NULL, dimension VARCHAR(128) NOT NULL, startPosX INT NOT NULL, startPosY INT NOT NULL, startPosZ INT NOT NULL, deleted SMALLINT NOT NULL DEFAULT 0, dateUpdated BIGINT NOT NULL, dateCreated BIGINT NOT NULL, PRIMARY KEY (raceTrackId))");
            s.execute("CREATE TABLE IF NOT EXISTS RaceGate (raceGateId VARCHAR(32) NOT NULL, raceTrackId VARCHAR(32) NOT NULL, gateIndex INT NOT NULL, dimension VARCHAR(128) NOT NULL, aPosX INT NOT NULL, aPosY INT NOT NULL, aPosZ INT NOT NULL, face INT NOT NULL, bPosX INT NOT NULL, bPosY INT NOT NULL, bPosZ INT NOT NULL, originX INT NOT NULL, originY INT NOT NULL, originZ INT NOT NULL, farthestX INT NOT NULL, farthestY INT NOT NULL, farthestZ INT NOT NULL, deleted SMALLINT NOT NULL DEFAULT 0, dateUpdated BIGINT NOT NULL, dateCreated BIGINT NOT NULL, PRIMARY KEY (raceGateId), UNIQUE (raceTrackId, gateIndex))");
            s.execute("CREATE TABLE IF NOT EXISTS RaceLap (raceLapId VARCHAR(32) NOT NULL, raceTrackId VARCHAR(32) NOT NULL, userId VARCHAR(32) NOT NULL, userName VARCHAR(32) NOT NULL, millis INT NOT NULL, version VARCHAR(16) NOT NULL, dateCreated BIGINT NOT NULL, PRIMARY KEY (raceLapId))");
        }
        Database.ensureIndex(c, "idx_racetrack_start", "RaceTrack", "startPosX, startPosZ, startPosY");
        Database.ensureIndex(c, "idx_racetrack_name", "RaceTrack", "name");
        Database.ensureIndex(c, "idx_racegate_track", "RaceGate", "raceTrackId");
        Database.ensureIndex(c, "idx_racelap_track_user", "RaceLap", "raceTrackId, userId, millis");
        return null;
    }

    private static void ensureIndex(Connection c, String name, String table, String columns) throws SQLException {
        try (Statement s = c.createStatement();){
            s.execute("CREATE INDEX IF NOT EXISTS " + name + " ON " + table + " (" + columns + ")");
        }
    }

    static {
        logInfo = s -> {};
    }

    @FunctionalInterface
    public static interface SqlFunction<T> {
        public T apply(Connection var1) throws Exception;
    }
}
