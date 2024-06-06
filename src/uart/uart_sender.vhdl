library IEEE;
use IEEE.STD_LOGIC_1164.all;
use IEEE.NUMERIC_STD.all;

entity pc_uart_send is
  port
  (
    clk            : in std_logic;
    reset          : in std_logic;
    data_in        : in std_logic_vector(7 downto 0);
    rs232_bit      : out std_logic;
    send_enable    : in std_logic;
    ready_for_data : out std_logic);
end pc_uart_send;

architecture Behavioral of pc_uart_send is

  constant fpga_frequency : integer := 12000000;
  constant bit_rate       : integer := 9600;

  -- internal state machine states
  type state_type is (READY, SENDING);
  signal state : state_type := READY;

  signal bit_sending_time : unsigned(19 downto 0);
  signal bit_sending_cnt  : unsigned(19 downto 0);
  signal bit_cnt          : unsigned(3 downto 0);
  signal data_int         : std_logic_vector(7 downto 0);

begin

  -- set duration of 1 bit in fpga-clks
  bit_sending_time <= (to_unsigned(fpga_frequency/bit_rate, bit_sending_time'length));

  SEND : process (clk)
  begin
    if rising_edge(clk) then
      if (reset = '1') then
        state           <= READY;
        bit_sending_cnt <= (others => '0');
        bit_cnt         <= (others => '0');
        rs232_bit       <= '1';
        data_int        <= data_in;
        ready_for_data  <= '1';
      else

        case state is
          when READY =>
            -- In READY state defaults values are set and the byte is sent when send_enable = 1
            if (send_enable = '1') then
              state          <= SENDING;
              ready_for_data <= '0';
            else
              ready_for_data <= '1';
            end if;

            bit_sending_cnt <= (others => '0');
            bit_cnt         <= (others => '0');
            data_int        <= data_in;

          when SENDING =>
            --start bit
            if (bit_cnt = 0) then
              rs232_bit <= '0';
              --data bits
            elsif (bit_cnt = 1) then
              rs232_bit <= data_int(0);
            elsif (bit_cnt = 2) then
              rs232_bit <= data_int(1);
            elsif (bit_cnt = 3) then
              rs232_bit <= data_int(2);
            elsif (bit_cnt = 4) then
              rs232_bit <= data_int(3);
            elsif (bit_cnt = 5) then
              rs232_bit <= data_int(4);
            elsif (bit_cnt = 6) then
              rs232_bit <= data_int(5);
            elsif (bit_cnt = 7) then
              rs232_bit <= data_int(6);
            elsif (bit_cnt = 8) then
              rs232_bit <= data_int(7);
            elsif (bit_cnt = 9) then
              -- stop bit
              rs232_bit <= '1';
            else
              state          <= READY;
              ready_for_data <= '1';
            end if;

            -- UART clock counter and bit selecter
            -- bit_sending_cnt counts from 0 to bit_sending_time,
            -- bit_cnt counts from 0 to 9 (s12345678)
            if (bit_sending_time <= bit_sending_cnt) then
              bit_sending_cnt      <= (others => '0');
              bit_cnt              <= bit_cnt + 1;
            else
              bit_sending_cnt <= bit_sending_cnt + 1;
              bit_cnt         <= bit_cnt + 0;
            end if;
        end case;
      end if;
    end if;
  end process;

end Behavioral;