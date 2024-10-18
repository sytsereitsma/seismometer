# from datetime import datetime
import os
import pickle
from datetime import datetime


NEWLINE_CHAR = 10  # "\n", but b"\n" != 10, so use int value
COMMA = 44  # ord(b",") == 44

class SeismodataReader:
    def __init__(self, filename):
        self.__index = []
        self.__file = open(filename, "rb")
        self.__load_index(filename)

        # Raw data has 7 columns, event data has 9 columns
        self.__is_raw_data = False
    
    @property
    def start_time(self):
        return self.__index[0][0]
    
    @property
    def end_time(self):
        return self.__index[-1][0]

    def get_data(self, start_time, end_time):
        if start_time < self.__index[0][0]:
            start_time = self.__index[0][0]
        
        start_index = self.__find_closest_timestamp(start_time)
        if start_index != 0:
            start_index -= 1

        end_index = self.__find_closest_timestamp(end_time)

        BLOCK_SIZE = 1024 * 1024
        start_offset = self.__index[start_index][1]
        end_offset = self.__index[end_index][1]
        remaining_bytes = end_offset - start_offset

        self.__file.seek(self.__index[start_index][1])
        data = bytes()
        start_line_pos = 0
        
        timestamps = []
        x = []
        y = []
        z = []
        remaining_data = bytes()
       
        while remaining_bytes > 0:
            bytes_to_read = BLOCK_SIZE if remaining_bytes > BLOCK_SIZE else remaining_bytes
            data = self.__file.read(bytes_to_read)

            if not data:
                # EOF
                break

            newline_pos = data.find(NEWLINE_CHAR, start_line_pos)
            while newline_pos != -1:
                if remaining_data:
                    timestamp, xv, yv, zv = self.__parse_line(remaining_data + data[:newline_pos])
                    remaining_data = bytes()
                else:
                    timestamp, xv, yv, zv = self.__parse_line(data[start_line_pos: newline_pos])

                if timestamp > end_time:
                    remaining_bytes = 0
                    break    

                if timestamp > start_time:
                    timestamps.append(datetime.fromtimestamp(timestamp / 1_000_000))
                    x.append(xv)
                    y.append(yv)
                    z.append(zv)

                start_line_pos = newline_pos + 1
                if start_line_pos >= len(data):
                    newline_pos = -1
                    data = bytes()
                else:
                    newline_pos = data.find(NEWLINE_CHAR, start_line_pos)
                    if newline_pos == -1 and start_line_pos < len(data):
                        remaining_data = data[start_line_pos:]
                        data = bytes()
                        start_line_pos = 0

        return timestamps, x, y, z
    
    def __parse_line(self, line: bytes()):
        timestamp, x, y, z = 0, 0, 0, 0
        line_find = line.find
        start_pos = 0
        comma_pos = line_find(COMMA)

        if comma_pos != -1 and start_pos < comma_pos:
            timestamp = int(line[start_pos:comma_pos])
            start_pos = comma_pos + 1
            comma_pos = line_find(COMMA, start_pos)

        if not self.__is_raw_data:
            # Skip microsec timestamp
            start_pos = comma_pos + 1
            comma_pos = line_find(COMMA, start_pos)

        if comma_pos != -1 and start_pos < comma_pos:
            x = int(line[start_pos:comma_pos])
            start_pos = comma_pos + 1
            comma_pos = line_find(COMMA, start_pos)

        if comma_pos != -1 and start_pos < comma_pos:
            y = int(line[start_pos:comma_pos])
            start_pos = comma_pos + 1
            comma_pos = line_find(COMMA, start_pos)

        if comma_pos != -1:
            z = int(line[start_pos:comma_pos])
            return timestamp, x, y, z
        
        return None

    def __load_index(self, filename):
        create_index = True

        index_filename = filename + ".index"
        if os.path.exists(index_filename):
            # Check if the data file has changed (by checking the modification time)
            data_timestamp = os.stat(filename).st_mtime
            index_timestamp = os.stat(index_filename).st_mtime

            if data_timestamp < index_timestamp:
                create_index = False
    
        if create_index:
            self.__create_index(index_filename)
        else:
            self.__index = pickle.load(open(index_filename, "rb"))
    
    def __get_timestamp_from_line(self, line):
        start_pos = 0
        comma_pos = line.find(b",")
        
        # if comma_pos != -1 and not self.__is_raw_data:
        #     # Event data has the us timestamp in the second column
        #     start_pos = comma_pos + 1
        #     comma_pos = line.find(COMMA, start_pos)

        if comma_pos != -1:
            try:
                return int(line[start_pos:comma_pos].decode())
            except ValueError as e:
                pass
        
        return None
    
    def __find_closest_timestamp(self, timestamp):
        # Binary search
        left = 0
        right = len(self.__index) - 1

        while left < right:
            mid = (left + right) // 2
            mid_timestamp = self.__index[mid][0]

            if mid_timestamp < timestamp:
                left = mid + 1
            else:
                right = mid

        return left


    def __create_index(self, index_filename):
        print("Creating index...")

        # Find the first newline to determine the file format
        self.__file.seek(0)
        line = self.__file.readline()
        num_columns = line.count(b",") + 1

        if num_columns != 7 and num_columns != 9:
            raise Exception("Invalid data format, expected 7, or 9 columns, got %d" % num_columns) 
        
        self.__is_raw_data = num_columns == 7
        
        self.__index = []
        self.__index.append((self.__get_timestamp_from_line(line), 0))

        keep_reading = True

        INDEX_INTERVAL = 2048
        BLOCK_SIZE = 256
        prev_data = None

        while keep_reading:
            keep_reading = False

            self.__file.seek(INDEX_INTERVAL, os.SEEK_CUR)
            file_pos = self.__file.tell()

            data = self.__file.read(BLOCK_SIZE)

            newline_pos = data.find(NEWLINE_CHAR)  # 10 is the ASCII code for newline
            if newline_pos != -1:
                newline_pos = data.find(NEWLINE_CHAR, newline_pos + 1)  # 10 is the ASCII code for newline
            
            # Ignore the first newline, we may have started looking for a newline in the middle of a line
            if newline_pos != -1:
                keep_reading = True
                timestamp_pos = newline_pos + 1
                file_pos += timestamp_pos

                timestamp = self.__get_timestamp_from_line(data[timestamp_pos:])
                if timestamp:
                    if timestamp < self.__index[-1][0]:
                        print(prev_data.decode())
                        print("-----------------")
                        print(data[timestamp_pos:].decode())
                        raise Exception("Timestamps are not in order, previous: %d, current: %d" % (self.__index[-1][0], timestamp))
                    else:
                        self.__index.append((timestamp, file_pos))
            
            prev_data = data[timestamp_pos:]

        pickle.dump(self.__index, open(index_filename, "wb"))
        print(f"Index created, first timestamp: {self.__index[0][0]}, last timestamp: {self.__index[-1][0]}")

if __name__ == "__main__":
    import time
    s = time.time()
    
    reader = SeismodataReader("seismodata.txt")
    start_time = reader.start_time + 10_000_000
    end_time = reader.end_time  # start_time + 10_000_000_000
    timestamps, x, y, z = reader.get_data(start_time, end_time)
    e = time.time()
    print(f"Done in {(e - s) * 1_000:.3f} ms {len(timestamps)}", )
    