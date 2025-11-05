#include <iostream>
using namespace std;

void swapintint(int& i,int& j) {
int temp;
temp = i;
i = j;
j = temp;
}

int main() {
int* t4 = new int[9];
t4[0] = 1.000000;
t4[1] = 6.000000;
t4[2] = 3.000000;
t4[3] = 98.000000;
t4[4] = 2.000000;
t4[5] = 4.000000;
t4[6] = 6.000000;
t4[7] = 1.000000;
t4[8] = 89.000000;
int* arr;
arr = t4;
int t6;
t6 = 0.000000;
for_start1:
int t7;
t7 = 8.000000 - t6;
if(!(t7)) goto for_end2;
int i;
i = t6;
int t8;
t8 = 0.000000;
for_start3:
int t9;
t9 = i - t8;
if(!(t9)) goto for_end4;
int j;
j = t8;
if_start5:
bool t10;
t10 = arr[j] > arr[i];
if(!(t10)) goto if_end6;
swapintint(arr[i],arr[j]);
if_end6:
int t12;
t12 = t8 + 0.000000;
t8 = t12;
goto for_start3;
for_end4:
int t13;
t13 = t6 + 0.000000;
t6 = t13;
goto for_start1;
for_end2:
int t14;
t14 = 0.000000;
for_start7:
int t15;
t15 = 9.000000 - t14;
if(!(t15)) goto for_end8;
i = t14;
std::cout<<arr[i];
std::cout<<' ';
int t16;
t16 = t14 + 0.000000;
t14 = t16;
goto for_start7;
for_end8:
}

