/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  net.minecraft.core.BlockPos
 *  net.minecraft.core.Direction
 */
package com.taylorgoolsby.fpvdrone.race;

import com.taylorgoolsby.fpvdrone.race.GetInteriorResult;
import com.taylorgoolsby.fpvdrone.race.RaceGate;
import com.taylorgoolsby.fpvdrone.race.RaceGateException;
import com.taylorgoolsby.fpvdrone.race.V3;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.function.BiPredicate;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;

public class RaceMath {
    public static final int MAX_PATH_LENGTH = 64;

    public static int countFixedAxes2(BlockPos a, BlockPos b) {
        int fixedAxes = 0;
        if (a.getX() == b.getX()) {
            ++fixedAxes;
        }
        if (a.getZ() == b.getZ()) {
            ++fixedAxes;
        }
        if (a.getY() == b.getY()) {
            ++fixedAxes;
        }
        return fixedAxes;
    }

    public static int countFixedAxes(BlockPos a, BlockPos b, BlockPos c) {
        int fixedAxes = 0;
        if (a.getX() == b.getX() && a.getX() == c.getX()) {
            ++fixedAxes;
        }
        if (a.getZ() == b.getZ() && a.getZ() == c.getZ()) {
            ++fixedAxes;
        }
        if (a.getY() == b.getY() && a.getY() == c.getY()) {
            ++fixedAxes;
        }
        return fixedAxes;
    }

    public static int getPlanarFixedDim(BlockPos a, BlockPos b, BlockPos c) throws RaceGateException {
        int nFixedDims = RaceMath.countFixedAxes(a, b, c);
        if (nFixedDims != 1) {
            throw new RaceGateException("Exactly 1 dimension should be fixed in order to be planar non-linear.");
        }
        if (a.getX() == b.getX() && a.getX() == c.getX()) {
            return 0;
        }
        if (a.getZ() == b.getZ() && a.getZ() == c.getZ()) {
            return 2;
        }
        if (a.getY() == b.getY() && a.getY() == c.getY()) {
            return 1;
        }
        throw new RaceGateException("Unable to find fixed dim.");
    }

    public static V3[] getPlanarDirections(int fixedDimension) {
        V3 right = V3.ZERO;
        V3 up = V3.ZERO;
        if (fixedDimension == 0) {
            right = new V3(0.0f, 0.0f, 1.0f);
            up = new V3(0.0f, 1.0f, 0.0f);
        } else if (fixedDimension == 1) {
            right = new V3(1.0f, 0.0f, 0.0f);
            up = new V3(0.0f, 0.0f, 1.0f);
        } else if (fixedDimension == 2) {
            right = new V3(1.0f, 0.0f, 0.0f);
            up = new V3(0.0f, 1.0f, 0.0f);
        }
        V3 normal = right.cross(up);
        return new V3[]{right, up, normal};
    }

    public static RaceGate loadGate(String dimension, BlockPos a, Direction face, BlockPos b, BiPredicate<String, BlockPos> checkSolid, RaceGate local) throws RaceGateException {
        boolean airTouching;
        BlockPos air = new BlockPos(a.getX() + face.getStepX(), a.getY() + face.getStepY(), a.getZ() + face.getStepZ());
        if (!checkSolid.test(dimension, a) || !checkSolid.test(dimension, b) || checkSolid.test(dimension, air)) {
            throw new RaceGateException("Starting blocks do not have the correct solidity.");
        }
        int nFixedDims = RaceMath.countFixedAxes(a, b, air);
        if (nFixedDims != 1) {
            throw new RaceGateException("Unable to determine axis-aligned dimension.");
        }
        int fixedDim = RaceMath.getPlanarFixedDim(a, b, air);
        int distB = RaceMath.manhattan(a, b);
        boolean bTouching = distB == 1 || distB == 2 && RaceMath.countFixedAxes2(a, b) == 1;
        boolean bl = airTouching = RaceMath.manhattan(a, air) == 1;
        if (!bTouching || !airTouching) {
            throw new RaceGateException("Starting blocks are not touching correctly.");
        }
        boolean bVisited = false;
        BlockPos currentSolid = a;
        BlockPos currentAir = RaceMath.rotateAirBlock(fixedDim, dimension, currentSolid, air, checkSolid);
        ArrayList<BlockPos> path = new ArrayList<BlockPos>(64);
        path.add(a);
        for (int length = 1; length < 65; ++length) {
            BlockPos[] results = RaceMath.stepGateBlock(fixedDim, dimension, currentSolid, currentAir, checkSolid);
            if (path.size() >= 2 && results[0].equals((Object)path.get(path.size() - 2))) {
                throw new RaceGateException("Dead end. Gate not closed.");
            }
            currentSolid = results[0];
            currentAir = results[1];
            currentAir = RaceMath.rotateAirBlock(fixedDim, dimension, currentSolid, currentAir, checkSolid);
            if (currentSolid.equals((Object)b)) {
                bVisited = true;
            }
            if (currentSolid.equals((Object)a)) break;
            path.add(currentSolid);
        }
        if (path.size() > 64) {
            throw new RaceGateException("Found path is greater than the allowed maximum.", "Gate is too big.");
        }
        if (path.size() == 64 && !currentSolid.equals((Object)a)) {
            throw new RaceGateException("Max iteration reached without wrapping back to starting point.", "Gate is too big.");
        }
        if (!bVisited) {
            throw new RaceGateException("B was not visited.");
        }
        BlockPos[] aabb = RaceMath.getAABB(path);
        BlockPos origin = aabb[0];
        BlockPos farthest = aabb[1];
        GetInteriorResult interiorResult = RaceMath.getInteriorBlocks(fixedDim, aabb, path);
        ArrayList<BlockPos> interior = interiorResult.blocks;
        int[] rowMin = interiorResult.rowMin;
        int[] rowMax = interiorResult.rowMax;
        boolean foundAir = false;
        for (BlockPos interiorBlock : interior) {
            if (checkSolid.test(dimension, interiorBlock)) {
                throw new RaceGateException("Found a solid interior block.", "Gate interior should not have solid blocks.");
            }
            if (!air.equals((Object)interiorBlock)) continue;
            foundAir = true;
        }
        if (!foundAir) {
            throw new RaceGateException("Air was not visited.", "You must click on the gate's interior face.");
        }
        V3[] planar = RaceMath.getPlanarDirections(fixedDim);
        RaceGate result = local == null ? new RaceGate(dimension, a, face, b) : local;
        result.setBoundaries(origin, farthest, rowMin, rowMax, planar[0], planar[1], planar[2], path);
        return result;
    }

    public static BlockPos[] getAABB(ArrayList<BlockPos> path) {
        int minX = Integer.MAX_VALUE;
        int minY = Integer.MAX_VALUE;
        int minZ = Integer.MAX_VALUE;
        int maxX = Integer.MIN_VALUE;
        int maxY = Integer.MIN_VALUE;
        int maxZ = Integer.MIN_VALUE;
        for (BlockPos e : path) {
            if (e.getX() < minX) {
                minX = e.getX();
            }
            if (e.getY() < minY) {
                minY = e.getY();
            }
            if (e.getZ() < minZ) {
                minZ = e.getZ();
            }
            if (e.getX() > maxX) {
                maxX = e.getX();
            }
            if (e.getY() > maxY) {
                maxY = e.getY();
            }
            if (e.getZ() <= maxZ) continue;
            maxZ = e.getZ();
        }
        return new BlockPos[]{new BlockPos(minX, minY, minZ), new BlockPos(maxX, maxY, maxZ)};
    }

    public static GetInteriorResult getInteriorBlocks(int fixedDimension, BlockPos[] maxAABB, ArrayList<BlockPos> path) throws RaceGateException {
        int rowIndex;
        int colIndex;
        int i;
        V3[] planar = RaceMath.getPlanarDirections(fixedDimension);
        V3 right = planar[0];
        V3 up = planar[1];
        V3 origin = new V3(maxAABB[0].getX(), maxAABB[0].getY(), maxAABB[0].getZ());
        V3 maxCorner = new V3(maxAABB[1].getX(), maxAABB[1].getY(), maxAABB[1].getZ());
        V3 span = maxCorner.sub(origin);
        int maxRowIndex = Math.round(Math.abs(span.dot(up)));
        int nRows = maxRowIndex + 1;
        int maxColIndex = Math.round(Math.abs(span.dot(right)));
        int nCols = maxColIndex + 1;
        ArrayList rows = new ArrayList();
        ArrayList cols = new ArrayList();
        for (i = 0; i < nRows; ++i) {
            rows.add(new HashMap());
        }
        for (i = 0; i < nCols; ++i) {
            cols.add(new HashMap());
        }
        int[] rowMin = new int[nRows];
        int[] rowMax = new int[nRows];
        for (int i2 = 0; i2 < nRows; ++i2) {
            rowMin[i2] = Integer.MAX_VALUE;
            rowMax[i2] = Integer.MIN_VALUE;
        }
        int[] colMin = new int[nCols];
        int[] colMax = new int[nCols];
        for (int i3 = 0; i3 < nCols; ++i3) {
            colMin[i3] = Integer.MAX_VALUE;
            colMax[i3] = Integer.MIN_VALUE;
        }
        for (BlockPos edgeBlock : path) {
            V3 local = new V3(edgeBlock.getX(), edgeBlock.getY(), edgeBlock.getZ()).sub(origin);
            int rowIndex2 = Math.round(Math.abs(local.dot(up)));
            colIndex = Math.round(Math.abs(local.dot(right)));
            if (colIndex < rowMin[rowIndex2]) {
                rowMin[rowIndex2] = colIndex;
            }
            if (colIndex > rowMax[rowIndex2]) {
                rowMax[rowIndex2] = colIndex;
            }
            if (rowIndex2 < colMin[colIndex]) {
                colMin[colIndex] = rowIndex2;
            }
            if (rowIndex2 > colMax[colIndex]) {
                colMax[colIndex] = rowIndex2;
            }
            ((HashMap)rows.get(rowIndex2)).put(colIndex, edgeBlock);
            ((HashMap)cols.get(colIndex)).put(rowIndex2, edgeBlock);
        }
        int[] outerRowMin = (int[])rowMin.clone();
        int[] outerRowMax = (int[])rowMax.clone();
        for (rowIndex = 0; rowIndex < nRows; ++rowIndex) {
            int colIndex2;
            BlockPos next;
            int count = 0;
            do {
                colIndex2 = rowMin[rowIndex];
                next = (BlockPos)((HashMap)rows.get(rowIndex)).get(colIndex2 + 1);
                ((HashMap)rows.get(rowIndex)).remove(colIndex2);
                if (next == null) continue;
                rowMin[rowIndex] = colIndex2 + 1;
            } while (next != null && ++count < 64);
            count = 0;
            do {
                colIndex2 = rowMax[rowIndex];
                next = (BlockPos)((HashMap)rows.get(rowIndex)).get(colIndex2 - 1);
                ((HashMap)rows.get(rowIndex)).remove(colIndex2);
                if (next == null) continue;
                rowMax[rowIndex] = colIndex2 - 1;
            } while (next != null && ++count < 64);
        }
        for (colIndex = 0; colIndex < nCols; ++colIndex) {
            int rowIndex3;
            BlockPos next;
            int count = 0;
            do {
                rowIndex3 = colMin[colIndex];
                next = (BlockPos)((HashMap)cols.get(colIndex)).get(rowIndex3 + 1);
                ((HashMap)cols.get(colIndex)).remove(rowIndex3);
                if (next == null) continue;
                colMin[colIndex] = rowIndex3 + 1;
            } while (next != null && ++count < 64);
            count = 0;
            do {
                rowIndex3 = colMax[colIndex];
                next = (BlockPos)((HashMap)cols.get(colIndex)).get(rowIndex3 - 1);
                ((HashMap)cols.get(colIndex)).remove(rowIndex3);
                if (next == null) continue;
                colMax[colIndex] = rowIndex3 - 1;
            } while (next != null && ++count < 64);
        }
        for (rowIndex = 0; rowIndex < nRows; ++rowIndex) {
            if (((HashMap)rows.get(rowIndex)).size() == 0) continue;
            throw new RaceGateException("Gate is not convex.", "Gate is not convex.");
        }
        for (colIndex = 0; colIndex < nCols; ++colIndex) {
            if (((HashMap)cols.get(colIndex)).size() == 0) continue;
            throw new RaceGateException("Gate is not convex.", "Gate is not convex.");
        }
        ArrayList<BlockPos> blocks = new ArrayList<BlockPos>();
        for (int row = 1; row < nRows - 1; ++row) {
            int currentRowMin = rowMin[row];
            int currentRowMax = rowMax[row];
            if (currentRowMin == Integer.MAX_VALUE || currentRowMax == Integer.MIN_VALUE) {
                throw new RaceGateException("Row without edge blocks.", "Gate is not closed.");
            }
            for (int col = currentRowMin + 1; col <= currentRowMax - 1; ++col) {
                V3 pos = origin.add(right.mul(col)).add(up.mul(row));
                blocks.add(new BlockPos(Math.round(pos.x), Math.round(pos.y), Math.round(pos.z)));
            }
        }
        GetInteriorResult result = new GetInteriorResult();
        result.blocks = blocks;
        result.rowMin = outerRowMin;
        result.rowMax = outerRowMax;
        return result;
    }

    private static BlockPos[] stepGateBlock(int fixedDimension, String worldDimension, BlockPos currentSolid, BlockPos currentAir, BiPredicate<String, BlockPos> checkSolid) throws RaceGateException {
        V3 normal = RaceMath.getPlanarDirections(fixedDimension)[2];
        V3 solidPos = new V3(currentSolid.getX(), currentSolid.getY(), currentSolid.getZ());
        V3 solidToAir = new V3(currentAir.getX() - currentSolid.getX(), currentAir.getY() - currentSolid.getY(), currentAir.getZ() - currentSolid.getZ());
        V3 nextDir = solidToAir.cross(normal).normalized();
        BlockPos diagonalPos = RaceMath.toBlockPos(solidPos.add(nextDir).add(solidToAir));
        boolean isDiagonalSolid = checkSolid.test(worldDimension, diagonalPos);
        BlockPos straightPos = RaceMath.toBlockPos(solidPos.add(nextDir));
        boolean isStraightSolid = checkSolid.test(worldDimension, straightPos);
        if (isDiagonalSolid) {
            return new BlockPos[]{diagonalPos, currentAir};
        }
        if (isStraightSolid) {
            return new BlockPos[]{straightPos, diagonalPos};
        }
        throw new RaceGateException("Unable to determine if we are in case 1 or 2.");
    }

    private static BlockPos rotateAirBlock(int fixedDimension, String worldDimension, BlockPos currentSolid, BlockPos currentAir, BiPredicate<String, BlockPos> checkSolid) throws RaceGateException {
        V3 normal = RaceMath.getPlanarDirections(fixedDimension)[2];
        V3 solidPos = new V3(currentSolid.getX(), currentSolid.getY(), currentSolid.getZ());
        BlockPos testAir = new BlockPos(currentAir.getX(), currentAir.getY(), currentAir.getZ());
        for (int i = 0; i < 3; ++i) {
            V3 solidToAir = new V3(testAir.getX() - currentSolid.getX(), testAir.getY() - currentSolid.getY(), testAir.getZ() - currentSolid.getZ());
            V3 nextDir = solidToAir.cross(normal).normalized();
            BlockPos diagonalPos = RaceMath.toBlockPos(solidPos.add(nextDir).add(solidToAir));
            boolean isDiagonalSolid = checkSolid.test(worldDimension, diagonalPos);
            BlockPos straightPos = RaceMath.toBlockPos(solidPos.add(nextDir));
            boolean isStraightSolid = checkSolid.test(worldDimension, straightPos);
            if (isDiagonalSolid || isStraightSolid) {
                return testAir;
            }
            testAir = straightPos;
        }
        throw new RaceGateException("Unable to find neighboring solid block.", "A single block cannot be a gate.");
    }

    private static int manhattan(BlockPos a, BlockPos b) {
        return Math.abs(a.getX() - b.getX()) + Math.abs(a.getY() - b.getY()) + Math.abs(a.getZ() - b.getZ());
    }

    private static BlockPos toBlockPos(V3 pos) {
        return new BlockPos((int)Math.floor(pos.x), (int)Math.floor(pos.y), (int)Math.floor(pos.z));
    }
}
